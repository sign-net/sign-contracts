#[cfg(not(feature = "library"))]
use crate::msg::{CollectionInfoResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{CollectionInfo, COLLECTION_INFO};
use crate::ContractError;
use cosmwasm_std::{entry_point, Addr, Coin, WasmMsg};
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Empty, Env, MessageInfo, StdResult};
use cw2::set_contract_version;
use cw721::{ContractInfoResponse, Cw721ReceiveMsg};
use cw721_base::msg::ExecuteMsg as Cw721ExecuteMsg;
use cw721_base::state::TokenInfo;
use cw721_base::{ContractError as BaseError, Cw721Contract, MintMsg};
use s1::{check_royalty_payment, OWNER_PERCENT, ROYALTY_FEE};
use s2::{check_payment, MIN_FEE};
use s_std::{FactoryExecuteMsg, Response, SubMsg, FACTORY, MULTI_SIG, NATIVE_DENOM};
use url::Url;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:s721";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const MAX_DESCRIPTION_LENGTH: u32 = 512;

type S721Contract<'a> = Cw721Contract<'a, Empty, Empty>;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // Creation fee paid to multisig
    let multisig = deps.api.addr_validate(MULTI_SIG)?;
    let mut msgs = vec![check_payment(&info, MIN_FEE, multisig)?];

    // Store creator address->s721 contract address to factory contract
    deps.api.addr_validate(&msg.collection_info.creator)?;
    let factory_msg = SubMsg::new(WasmMsg::Execute {
        contract_addr: FACTORY.to_string(),
        msg: to_binary(&FactoryExecuteMsg::AddS721 {
            from: msg.clone().collection_info.creator,
        })?,
        funds: vec![],
    });
    msgs.push(factory_msg);

    // cw721 instantiation
    let contract_info = ContractInfoResponse {
        name: msg.name,
        symbol: msg.symbol,
    };
    S721Contract::default()
        .contract_info
        .save(deps.storage, &contract_info)?;

    let minter = deps.api.addr_validate(&msg.minter)?;
    S721Contract::default().minter.save(deps.storage, &minter)?;

    // s721 instantiation
    if msg.collection_info.description.len() > MAX_DESCRIPTION_LENGTH as usize {
        return Err(ContractError::DescriptionTooLong {});
    }

    Url::parse(&msg.collection_info.image)?;

    if let Some(ref external_link) = msg.collection_info.external_link {
        Url::parse(external_link)?;
    }

    deps.api
        .addr_validate(&msg.collection_info.royalty_address)?;

    let collection_info = CollectionInfo {
        creator: msg.collection_info.creator,
        description: msg.collection_info.description,
        image: msg.collection_info.image,
        external_link: msg.collection_info.external_link,
        royalty_address: msg.collection_info.royalty_address,
    };

    COLLECTION_INFO.save(deps.storage, &collection_info)?;

    let mut rsp = Response::default();
    rsp.messages = msgs;

    Ok(rsp
        .add_attribute("action", "instantiate")
        .add_attribute("contract_name", CONTRACT_NAME)
        .add_attribute("contract_version", CONTRACT_VERSION)
        .add_attribute("creation_fee", info.funds[0].to_string())
        .add_attribute("payment_address", MULTI_SIG))
}

/// To mitigate clippy::too_many_arguments warning
pub struct ExecuteEnv<'a> {
    deps: DepsMut<'a>,
    env: Env,
    info: MessageInfo,
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let env = ExecuteEnv { deps, env, info };

    match msg {
        ExecuteMsg::TransferNft {
            recipient,
            token_id,
        } => execute_transfer_nft(env, recipient, token_id),
        ExecuteMsg::SendNft {
            contract,
            token_id,
            msg,
        } => execute_send_nft(env, contract, token_id, msg),
        ExecuteMsg::Mint(msg) => execute_mint(env, msg),
        _ => {
            match S721Contract::default().execute(
                env.deps,
                env.env,
                env.info,
                Cw721ExecuteMsg::from(msg),
            ) {
                Ok(res) => {
                    // Messsages not required here
                    let mut new_res = Response::new();
                    new_res.attributes = res.attributes;
                    new_res.events = res.events;
                    new_res.data = res.data;
                    Ok(new_res)
                }
                Err(err) => match err {
                    BaseError::Std(from) => Err(ContractError::Std(from)),
                    BaseError::Unauthorized {} => Err(ContractError::Unauthorized {}),
                    BaseError::Expired {} => Err(ContractError::Expired {}),
                    BaseError::Claimed {} => Err(ContractError::Claimed {}),
                    BaseError::ApprovalNotFound { spender } => {
                        Err(ContractError::ApprovalNotFound { spender })
                    }
                },
            }
        }
    }
}

pub fn execute_transfer_nft(
    env: ExecuteEnv,
    recipient: String,
    token_id: String,
) -> Result<Response, ContractError> {
    let ExecuteEnv { deps, env, info } = env;
    let royalty_address = COLLECTION_INFO.load(deps.storage)?.royalty_address;

    let msgs = _transfer_nft(deps, &env, &info, &recipient, &token_id, &royalty_address)?;
    let mut rsp = Response::new()
        .add_attribute("action", "transfer_nft")
        .add_attribute("sender", info.sender)
        .add_attribute("recipient", recipient)
        .add_attribute("token_id", token_id)
        .add_attribute(
            "royalty_fee",
            Coin::new(ROYALTY_FEE, NATIVE_DENOM).to_string(),
        )
        .add_attribute("royalty_address", royalty_address)
        .add_attribute("royalty_share", OWNER_PERCENT.to_string());
    rsp.messages = msgs;
    Ok(rsp)
}

pub fn execute_send_nft(
    env: ExecuteEnv,
    contract: String,
    token_id: String,
    msg: Binary,
) -> Result<Response, ContractError> {
    let ExecuteEnv { deps, env, info } = env;

    // Transfer token
    let royalty_address = COLLECTION_INFO.load(deps.storage)?.royalty_address;
    let mut msgs = _transfer_nft(deps, &env, &info, &contract, &token_id, &royalty_address)?;
    msgs.push(SubMsg::new(
        Cw721ReceiveMsg {
            sender: info.sender.to_string(),
            token_id: token_id.clone(),
            msg,
        }
        .into_cosmos_msg(contract.clone())?,
    ));

    let mut rsp = Response::new()
        .add_attribute("action", "send_nft")
        .add_attribute("sender", info.sender)
        .add_attribute("recipient", contract)
        .add_attribute("token_id", token_id)
        .add_attribute(
            "royalty_fee",
            Coin::new(ROYALTY_FEE, NATIVE_DENOM).to_string(),
        )
        .add_attribute("royalty_address", royalty_address)
        .add_attribute("royalty_share", OWNER_PERCENT.to_string());
    rsp.messages = msgs;

    // Send message
    Ok(rsp)
}

pub fn execute_mint(env: ExecuteEnv, msg: MintMsg<Empty>) -> Result<Response, ContractError> {
    let ExecuteEnv { deps, env: _, info } = env;
    let minter = S721Contract::default().minter.load(deps.storage)?;

    if info.sender != minter {
        return Err(ContractError::Unauthorized {});
    }

    // Minting fee paid to multisig
    let multisig = Addr::unchecked(MULTI_SIG);
    let msgs = vec![check_payment(&info, MIN_FEE, multisig)?];

    // create the token
    let token = TokenInfo {
        owner: deps.api.addr_validate(&msg.owner)?,
        approvals: vec![],
        token_uri: msg.token_uri,
        extension: msg.extension,
    };
    S721Contract::default()
        .tokens
        .update(deps.storage, &msg.token_id, |old| match old {
            Some(_) => Err(ContractError::Claimed {}),
            None => Ok(token),
        })?;

    S721Contract::default().increment_tokens(deps.storage)?;

    let mut rsp = Response::new()
        .add_attribute("action", "mint")
        .add_attribute("minter", info.sender)
        .add_attribute("owner", msg.owner)
        .add_attribute("token_id", msg.token_id)
        .add_attribute("mint_fee", info.funds[0].to_string())
        .add_attribute("payment_address", MULTI_SIG);
    rsp.messages = msgs;

    Ok(rsp)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::CollectionInfo {} => to_binary(&query_config(deps)?),
        _ => S721Contract::default().query(deps, env, msg.into()),
    }
}

fn query_config(deps: Deps) -> StdResult<CollectionInfoResponse> {
    let CollectionInfo {
        creator,
        description,
        image,
        external_link,
        royalty_address,
    } = COLLECTION_INFO.load(deps.storage)?;

    Ok(CollectionInfoResponse {
        creator,
        description,
        image,
        external_link,
        royalty_address,
        factory_address: FACTORY.to_string(),
        multisig: MULTI_SIG.to_string(),
        min_fee: Coin::new(MIN_FEE, NATIVE_DENOM),
        royalty_fee: Coin::new(ROYALTY_FEE, NATIVE_DENOM),
        royalty_share: OWNER_PERCENT,
    })
}

/**********************************HELPERS*************************************/

fn _transfer_nft(
    deps: DepsMut,
    env: &Env,
    info: &MessageInfo,
    recipient: &str,
    token_id: &str,
    royalty_address: &str,
) -> Result<Vec<SubMsg>, ContractError> {
    let mut token = S721Contract::default()
        .tokens
        .load(deps.storage, token_id)?;

    // ensure we have permissions
    _check_can_send(deps.as_ref(), env, info, &token)?;

    // Royalty payment
    let msgs = check_royalty_payment(
        info,
        ROYALTY_FEE,
        // Has been validated at contract instantiation
        Addr::unchecked(royalty_address),
    )?;

    // set owner and remove existing approvals
    token.owner = deps.api.addr_validate(recipient)?;
    token.approvals = vec![];
    S721Contract::default()
        .tokens
        .save(deps.storage, token_id, &token)?;
    Ok(msgs)
}

/// returns true iff the sender can transfer ownership of the token
fn _check_can_send(
    deps: Deps,
    env: &Env,
    info: &MessageInfo,
    token: &TokenInfo<Empty>,
) -> Result<(), ContractError> {
    // owner can send
    if token.owner == info.sender {
        return Ok(());
    }

    // any non-expired token approval can send
    if token
        .approvals
        .iter()
        .any(|apr| apr.spender == info.sender && !apr.is_expired(&env.block))
    {
        return Ok(());
    }

    // operator can send
    let op = S721Contract::default()
        .operators
        .may_load(deps.storage, (&token.owner, &info.sender))?;
    match op {
        Some(ex) => {
            if ex.is_expired(&env.block) {
                Err(ContractError::Unauthorized {})
            } else {
                Ok(())
            }
        }
        None => Err(ContractError::Unauthorized {}),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::state::CollectionInfo;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary, BankMsg, WasmMsg};
    use cw721::{Cw721Query, NftInfoResponse, OwnerOfResponse};
    use s_std::error::FeeError;
    use s_std::{create_fund_community_pool_msg, CosmosMsg, SubMsg, NATIVE_DENOM};

    fn setup_contract(deps: DepsMut<'_>, minter: String, creator: String) {
        let collection = String::from("collection0");
        let info = mock_info(minter.as_str(), &coins(MIN_FEE, NATIVE_DENOM));
        let msg = InstantiateMsg {
            name: collection,
            symbol: String::from("DOC"),
            minter,
            collection_info: CollectionInfo {
                creator: creator.clone(),
                description: String::from("Document"),
                image: "https://example.com/image.png".to_string(),
                external_link: Some("https://example.com/external.html".to_string()),
                royalty_address: creator,
            },
        };
        instantiate(deps, mock_env(), info, msg).unwrap();
    }

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();
        let collection = String::from("collection0");
        let minter = String::from("minter");
        let creator = String::from("creator");
        let info = mock_info(minter.as_str(), &coins(15_000_000, NATIVE_DENOM));

        let msg = InstantiateMsg {
            name: collection,
            minter: minter.clone(),
            symbol: String::from("DOC"),
            collection_info: CollectionInfo {
                creator: creator.clone(),
                description: String::from("Document"),
                image: "https://example.com/image.png".to_string(),
                external_link: Some("https://example.com/external.html".to_string()),
                royalty_address: creator.clone(),
            },
        };

        // Error: Insufficient minting fee
        assert!(matches!(
            instantiate(deps.as_mut(), mock_env(), info, msg.clone()),
            Err(ContractError::Fee(FeeError::InsufficientFee(
                MIN_FEE, 15_000_000
            )))
        ));

        let info = mock_info(minter.as_str(), &coins(MIN_FEE, NATIVE_DENOM));

        // success
        let bank_msg = SubMsg::new(BankMsg::Send {
            to_address: MULTI_SIG.to_string(),
            amount: coins(MIN_FEE, NATIVE_DENOM.to_string()),
        });
        let factory_msg = SubMsg::new(WasmMsg::Execute {
            contract_addr: FACTORY.to_string(),
            msg: to_binary(&FactoryExecuteMsg::AddS721 {
                from: creator.clone(),
            })
            .unwrap(),
            funds: vec![],
        });
        let mut rsp = Response::new()
            .add_attribute("action", "instantiate")
            .add_attribute("contract_name", CONTRACT_NAME)
            .add_attribute("contract_version", CONTRACT_VERSION)
            .add_attribute("creation_fee", info.funds[0].to_string())
            .add_attribute("payment_address", MULTI_SIG);
        rsp.messages = vec![bank_msg, factory_msg];
        assert_eq!(
            instantiate(deps.as_mut(), mock_env(), info, msg).unwrap(),
            rsp
        );

        // let's query the collection info
        let res = query(deps.as_ref(), mock_env(), QueryMsg::CollectionInfo {}).unwrap();
        let value: CollectionInfoResponse = from_binary(&res).unwrap();
        assert_eq!("https://example.com/image.png", value.image);
        assert_eq!("Document", value.description);
        assert_eq!(
            "https://example.com/external.html",
            value.external_link.unwrap()
        );
        assert_eq!(creator, value.royalty_address);
    }

    #[test]
    fn test_mint() {
        let mut deps = mock_dependencies();
        let minter = String::from("minter");
        let user = String::from("user");
        setup_contract(deps.as_mut(), minter.clone(), String::from("creator"));
        let contract = S721Contract::default();

        let token_id = "token".to_string();
        let token_uri = "https://example.com/token_uri".to_string();

        let mint_msg = ExecuteMsg::Mint(MintMsg::<Empty> {
            token_id: token_id.clone(),
            owner: user.clone(),
            token_uri: Some(token_uri.clone()),
            extension: Empty {},
        });

        // Error: only contract creator is authorised to mint
        let rsp = execute(
            deps.as_mut(),
            mock_env(),
            mock_info(user.as_str(), &[]),
            mint_msg.clone(),
        );
        assert!(matches!(rsp, Err(ContractError::Unauthorized {})));

        // Error: insufficient fee to mint
        let rsp = execute(
            deps.as_mut(),
            mock_env(),
            mock_info(&minter, &coins(15_000_000, NATIVE_DENOM)),
            mint_msg.clone(),
        );
        assert!(matches!(
            rsp,
            Err(ContractError::Fee(FeeError::InsufficientFee(
                MIN_FEE, 15_000_000
            )))
        ));

        // mint to user1
        let info = mock_info(&minter, &coins(MIN_FEE, NATIVE_DENOM));
        let bank_msg = SubMsg::new(BankMsg::Send {
            to_address: MULTI_SIG.to_string(),
            amount: coins(MIN_FEE, NATIVE_DENOM.to_string()),
        });
        let mut rsp = Response::new()
            .add_attribute("action", "mint")
            .add_attribute("minter", minter.clone())
            .add_attribute("owner", user.clone())
            .add_attribute("token_id", token_id.clone())
            .add_attribute("mint_fee", info.funds[0].to_string())
            .add_attribute("payment_address", MULTI_SIG);
        rsp.messages = vec![bank_msg];
        assert_eq!(
            execute(deps.as_mut(), mock_env(), info, mint_msg,).unwrap(),
            rsp
        );
        assert_eq!(1, contract.token_count.load(&deps.storage).unwrap());

        // nft info is correct
        let info = contract.nft_info(deps.as_ref(), token_id.clone()).unwrap();
        assert_eq!(
            info,
            NftInfoResponse::<Empty> {
                token_uri: Some(token_uri),
                extension: Empty {},
            }
        );

        // owner info is correct
        let owner = contract
            .owner_of(deps.as_ref(), mock_env(), token_id.clone(), true)
            .unwrap();
        assert_eq!(
            owner,
            OwnerOfResponse {
                owner: user,
                approvals: vec![],
            }
        );

        // Cannot mint same token_id again
        let mint_msg = ExecuteMsg::Mint(MintMsg::<Empty> {
            token_id: token_id.clone(),
            owner: String::from("user2"),
            token_uri: None,
            extension: Empty {},
        });
        assert!(matches!(
            execute(
                deps.as_mut(),
                mock_env(),
                mock_info(&minter, &coins(MIN_FEE, NATIVE_DENOM)),
                mint_msg,
            ),
            Err(ContractError::Claimed {})
        ));

        let tokens = contract.all_tokens(deps.as_ref(), None, None).unwrap();
        assert_eq!(1, tokens.tokens.len());
        assert_eq!(vec![token_id], tokens.tokens);
    }

    #[test]
    fn test_transfer() {
        let mut deps = mock_dependencies();
        let minter = String::from("minter");
        let creator = String::from("creator");
        let user1 = String::from("user1");
        let user2 = String::from("user2");

        setup_contract(deps.as_mut(), minter.clone(), creator.clone());

        // mint
        let token_id = "token".to_string();
        let token_uri = "https://example.com/token_uri".to_string();
        let mint_msg = ExecuteMsg::Mint(MintMsg::<Empty> {
            token_id: token_id.clone(),
            owner: user1.clone(),
            token_uri: Some(token_uri),
            extension: Empty {},
        });
        execute(
            deps.as_mut(),
            mock_env(),
            mock_info(&minter, &coins(MIN_FEE, NATIVE_DENOM)),
            mint_msg,
        )
        .unwrap();

        // random cannot transfer
        let transfer_msg = ExecuteMsg::TransferNft {
            recipient: user2.clone(),
            token_id: token_id.clone(),
        };
        assert!(matches!(
            execute(
                deps.as_mut(),
                mock_env(),
                mock_info("random", &[]),
                transfer_msg.clone()
            ),
            Err(ContractError::Unauthorized {})
        ));

        // Below transfer royalty fee
        assert!(matches!(
            execute(
                deps.as_mut(),
                mock_env(),
                mock_info(user1.as_ref(), &coins(500u128, NATIVE_DENOM)),
                transfer_msg.clone(),
            ),
            Err(ContractError::Fee(FeeError::InsufficientFee(
                ROYALTY_FEE,
                500u128
            )))
        ));

        // success transfer
        let info = mock_info(user1.as_ref(), &coins(ROYALTY_FEE, NATIVE_DENOM));
        let bank_msg = SubMsg::new(BankMsg::Send {
            to_address: creator.clone(),
            amount: coins(950u128, NATIVE_DENOM.to_string()),
        });
        let community_msg =
            SubMsg::new(create_fund_community_pool_msg(coins(50u128, NATIVE_DENOM)));
        let mut rsp = Response::new()
            .add_attribute("action", "transfer_nft")
            .add_attribute("sender", user1)
            .add_attribute("recipient", user2.clone())
            .add_attribute("token_id", token_id.clone())
            .add_attribute("royalty_fee", info.funds[0].to_string())
            .add_attribute("royalty_address", creator)
            .add_attribute("royalty_share", OWNER_PERCENT.to_string());
        rsp.messages = vec![bank_msg, community_msg];
        assert_eq!(
            execute(deps.as_mut(), mock_env(), info, transfer_msg,).unwrap(),
            rsp
        );
        // owner info is correct
        let owner = S721Contract::default()
            .owner_of(deps.as_ref(), mock_env(), token_id, true)
            .unwrap();
        assert_eq!(
            owner,
            OwnerOfResponse {
                owner: user2,
                approvals: vec![],
            }
        );
    }

    #[test]
    fn test_send() {
        let mut deps = mock_dependencies();
        let minter = String::from("minter");
        let creator = String::from("creator");
        let user1 = String::from("user1");
        let contract = String::from("contract");

        setup_contract(deps.as_mut(), minter.clone(), creator.clone());

        // mint
        let token_id = "token".to_string();
        let token_uri = "https://example.com/token_uri".to_string();
        let mint_msg = ExecuteMsg::Mint(MintMsg::<Empty> {
            token_id: token_id.clone(),
            owner: user1.clone(),
            token_uri: Some(token_uri),
            extension: Empty {},
        });
        execute(
            deps.as_mut(),
            mock_env(),
            mock_info(&minter, &coins(MIN_FEE, NATIVE_DENOM)),
            mint_msg,
        )
        .unwrap();

        // random cannot transfer
        let msg = to_binary("You now have the melting power").unwrap();
        let send_msg = ExecuteMsg::SendNft {
            contract: contract.clone(),
            token_id: token_id.clone(),
            msg: msg.clone(),
        };
        assert!(matches!(
            execute(
                deps.as_mut(),
                mock_env(),
                mock_info("random", &[]),
                send_msg.clone()
            ),
            Err(ContractError::Unauthorized {})
        ));

        // Below transfer royalty fee
        assert!(matches!(
            execute(
                deps.as_mut(),
                mock_env(),
                mock_info(user1.as_ref(), &coins(500u128, NATIVE_DENOM)),
                send_msg.clone(),
            ),
            Err(ContractError::Fee(FeeError::InsufficientFee(
                ROYALTY_FEE,
                500u128
            )))
        ));

        // success send
        let bank_msg = SubMsg::new(BankMsg::Send {
            to_address: creator.clone(),
            amount: coins(950u128, NATIVE_DENOM.to_string()),
        });
        let community_msg =
            SubMsg::new(create_fund_community_pool_msg(coins(50u128, NATIVE_DENOM)));

        let payload = Cw721ReceiveMsg {
            sender: user1.clone(),
            token_id: token_id.clone(),
            msg,
        };
        let expected = payload.into_cosmos_msg(contract.clone()).unwrap();
        // ensure expected serializes as we think it should
        match &expected {
            CosmosMsg::Wasm(WasmMsg::Execute { contract_addr, .. }) => {
                assert_eq!(contract_addr, &contract)
            }
            m => panic!("Unexpected message type: {:?}", m),
        }
        let info = mock_info(user1.as_ref(), &coins(ROYALTY_FEE, NATIVE_DENOM));
        let mut rsp = Response::new()
            .add_attribute("action", "send_nft")
            .add_attribute("sender", user1)
            .add_attribute("recipient", contract.clone())
            .add_attribute("token_id", token_id.clone())
            .add_attribute("royalty_fee", info.funds[0].to_string())
            .add_attribute("royalty_address", creator)
            .add_attribute("royalty_share", OWNER_PERCENT.to_string());

        rsp.messages = vec![bank_msg, community_msg, SubMsg::new(expected)];
        assert_eq!(
            execute(deps.as_mut(), mock_env(), info, send_msg,).unwrap(),
            rsp
        );

        // owner info is correct
        let owner = S721Contract::default()
            .owner_of(deps.as_ref(), mock_env(), token_id, true)
            .unwrap();
        assert_eq!(
            owner,
            OwnerOfResponse {
                owner: contract,
                approvals: vec![],
            }
        );
    }
}
