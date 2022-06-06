use crate::error::{ContractError, FeeError};
use crate::msg::{ExecuteMsg, QueryMsg, RoyaltyInfo as RoyaltyInfoMsg, TokenInfoResponse};
use crate::state::{RoyaltyInfo, TokenInfo, BALANCES, MINTER, TOKENS};
use cosmwasm_std::{entry_point, to_binary, Addr, Binary, Deps, SubMsg, Uint128};
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, StdResult};
use cw1155::{
    Cw1155BatchReceiveMsg, Cw1155ExecuteMsg, Cw1155QueryMsg, Cw1155ReceiveMsg, TokenId,
    TransferEvent,
};
use cw1155_base::contract::{execute as base_execute, query as base_query};

use cw1155_base::msg::InstantiateMsg;
use cw1155_base::ContractError as BaseError;
use cw2::set_contract_version;
use cw_utils::{must_pay, Event};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:s1155";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const NATIVE_DENOM: &str = "usign";
const CREATION_FEE: u128 = 5_000_000;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let minter = deps.api.addr_validate(&msg.minter)?;
    MINTER.save(deps.storage, &minter)?;
    Ok(Response::default())
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
        ExecuteMsg::Mint {
            to,
            token_id,
            value,
            token_info,
            msg,
        } => execute_mint(env, to, token_id, value, token_info, msg),
        ExecuteMsg::BatchMint {
            to,
            batch,
            token_info_batch,
            msg,
        } => execute_batch_mint(env, to, batch, token_info_batch, msg),
        _ => {
            let result = base_execute(env.deps, env.env, env.info, Cw1155ExecuteMsg::from(msg));
            match result {
                Ok(res) => Ok(res),
                Err(err) => match err {
                    BaseError::Std(from) => Err(ContractError::Std(from)),
                    BaseError::Unauthorized {} => Err(ContractError::Unauthorized {}),
                    BaseError::Expired {} => Err(ContractError::Expired {}),
                },
            }
        }
    }
}

pub fn execute_mint(
    env: ExecuteEnv,
    to: String,
    token_id: TokenId,
    amount: Uint128,
    token_info: TokenInfo<RoyaltyInfoMsg>,
    msg: Option<Binary>,
) -> Result<Response, ContractError> {
    let ExecuteEnv { mut deps, info, .. } = env;

    let payment = must_pay(&info, NATIVE_DENOM)?;
    if payment.u128() < CREATION_FEE {
        return Err(ContractError::Fee(FeeError::InsufficientFee(
            CREATION_FEE,
            payment.u128(),
        )));
    };

    let to_addr = deps.api.addr_validate(&to)?;

    if info.sender != MINTER.load(deps.storage)? {
        return Err(ContractError::Unauthorized {});
    }

    let mut rsp = Response::default();

    let TokenInfo {
        document_uri,
        image_uri,
        token_uri,
        royalty_info,
    } = token_info;

    // Validate royalty info
    let royalty_info_result = match &royalty_info {
        Some(royalty_info) => Some(RoyaltyInfo {
            payment_address: deps
                .api
                .addr_validate(royalty_info.payment_address.as_str())?,
            share: royalty_info.share_validate()?,
        }),
        None => None,
    };

    let event = execute_transfer_inner(&mut deps, None, Some(&to_addr), &token_id, amount)?;
    event.add_attributes(&mut rsp);

    if let Some(msg) = msg {
        rsp.messages = vec![SubMsg::new(
            Cw1155ReceiveMsg {
                operator: info.sender.to_string(),
                from: None,
                amount,
                token_id: token_id.clone(),
                msg,
            }
            .into_cosmos_msg(to)?,
        )]
    }

    // insert if not exist
    if !TOKENS.has(deps.storage, &token_id) {
        // we must save some valid data here
        TOKENS.save(
            deps.storage,
            &token_id,
            &TokenInfo {
                document_uri,
                image_uri,
                token_uri,
                royalty_info: royalty_info_result,
            },
        )?;
    }

    Ok(rsp)
}

pub fn execute_batch_mint(
    env: ExecuteEnv,
    to: String,
    batch: Vec<(TokenId, Uint128)>,
    token_info_batch: Vec<TokenInfo<RoyaltyInfoMsg>>,
    msg: Option<Binary>,
) -> Result<Response, ContractError> {
    let ExecuteEnv { mut deps, info, .. } = env;
    if info.sender != MINTER.load(deps.storage)? {
        return Err(ContractError::Unauthorized {});
    }

    let to_addr = deps.api.addr_validate(&to)?;

    let mut rsp = Response::default();

    if batch.len() != token_info_batch.len() {
        return Err(ContractError::TokenInfoNotFound {});
    }

    let payment = must_pay(&info, NATIVE_DENOM)?;
    let total_fees =
        CREATION_FEE * u128::try_from(batch.len()).expect("Unable to calculate total fee");
    if payment.u128() < total_fees {
        return Err(ContractError::Fee(FeeError::InsufficientFee(
            CREATION_FEE,
            payment.u128(),
        )));
    };

    for (pos, (token_id, amount)) in batch.iter().enumerate() {
        let event = execute_transfer_inner(&mut deps, None, Some(&to_addr), token_id, *amount)?;
        event.add_attributes(&mut rsp);

        let token_info = token_info_batch
            .get(pos)
            .ok_or(ContractError::TokenInfoNotFound {})?;

        let TokenInfo {
            document_uri,
            image_uri,
            token_uri,
            royalty_info,
        } = token_info;

        // Validate royalty info
        let royalty_info_result = match royalty_info {
            Some(royalty_info) => Some(RoyaltyInfo {
                payment_address: deps
                    .api
                    .addr_validate(royalty_info.payment_address.as_str())?,
                share: royalty_info.share_validate()?,
            }),
            None => None,
        };

        // insert if not exist
        if !TOKENS.has(deps.storage, token_id) {
            // we must save some valid data here
            TOKENS.save(
                deps.storage,
                token_id,
                &TokenInfo {
                    document_uri: document_uri.to_string(),
                    image_uri: image_uri.to_string(),
                    token_uri: token_uri.to_string(),
                    royalty_info: royalty_info_result,
                },
            )?;
        }
    }

    if let Some(msg) = msg {
        rsp.messages = vec![SubMsg::new(
            Cw1155BatchReceiveMsg {
                operator: info.sender.to_string(),
                from: None,
                batch,
                msg,
            }
            .into_cosmos_msg(to)?,
        )]
    };

    Ok(rsp)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::TokenInfo { token_id } => to_binary(&query_config(deps, token_id)?),
        _ => base_query(deps, env, Cw1155QueryMsg::from(msg)),
    }
}

fn query_config(deps: Deps, token_id: TokenId) -> StdResult<TokenInfoResponse> {
    let TokenInfo {
        image_uri,
        document_uri,
        token_uri,
        royalty_info,
    } = TOKENS.load(deps.storage, &token_id)?;
    let royalty_info_result: Option<RoyaltyInfoMsg> = match royalty_info {
        Some(royalty_info) => Some(RoyaltyInfoMsg {
            payment_address: royalty_info.payment_address.to_string(),
            share: royalty_info.share,
        }),
        None => None,
    };
    Ok(TokenInfoResponse {
        image_uri,
        document_uri,
        token_uri,
        royalty_info: royalty_info_result,
    })
}

/// When from is None: mint new coins
/// When to is None: burn coins
/// When both are None: no token balance is changed, pointless but valid
///
/// Make sure permissions are checked before calling this.
fn execute_transfer_inner<'a>(
    deps: &'a mut DepsMut,
    from: Option<&'a Addr>,
    to: Option<&'a Addr>,
    token_id: &'a str,
    amount: Uint128,
) -> Result<TransferEvent<'a>, ContractError> {
    if let Some(from_addr) = from {
        BALANCES.update(
            deps.storage,
            (from_addr, token_id),
            |balance: Option<Uint128>| -> StdResult<_> {
                Ok(balance.unwrap_or_default().checked_sub(amount)?)
            },
        )?;
    }

    if let Some(to_addr) = to {
        BALANCES.update(
            deps.storage,
            (to_addr, token_id),
            |balance: Option<Uint128>| -> StdResult<_> {
                Ok(balance.unwrap_or_default().checked_add(amount)?)
            },
        )?;
    }

    Ok(TransferEvent {
        from: from.map(|x| x.as_ref()),
        to: to.map(|x| x.as_ref()),
        token_id,
        amount,
    })
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{
        coins, from_binary,
        testing::{mock_dependencies, mock_env, mock_info},
        to_binary, Decimal,
    };
    use cw1155::BalanceResponse;
    use cw_utils::PaymentError;

    use super::*;

    #[test]
    fn test_mint() {
        let minter = String::from("minter");
        let user1 = String::from("user1");

        let token1 = "token1".to_owned();
        let token_info1 = TokenInfo {
            image_uri: "https://example.com/image1".to_string(),
            document_uri: "https://example.com/document1".to_string(),
            token_uri: "https://example.com/token_uri1".to_string(),
            royalty_info: None,
        };

        let token2 = "token2".to_owned();
        let token_info2 = TokenInfo {
            image_uri: "https://example.com/image1".to_string(),
            document_uri: "https://example.com/document1".to_string(),
            token_uri: "https://example.com/token_uri1".to_string(),
            royalty_info: Some(RoyaltyInfoMsg {
                payment_address: minter.clone(),
                share: Decimal::percent(5),
            }),
        };

        let mut deps = mock_dependencies();
        // instantiate contract for "minter"
        let msg = InstantiateMsg {
            minter: minter.clone(),
        };
        let res = instantiate(deps.as_mut(), mock_env(), mock_info("operator", &[]), msg).unwrap();
        assert_eq!(0, res.messages.len());

        let mint_msg = ExecuteMsg::Mint {
            to: user1.clone(),
            token_id: token1.clone(),
            value: 1u64.into(),
            token_info: token_info1.clone(),
            msg: None,
        };
        // invalid mint, user1 don't mint permission on "minter" contract
        assert!(matches!(
            execute(
                deps.as_mut(),
                mock_env(),
                mock_info(user1.as_ref(), &coins(CREATION_FEE, NATIVE_DENOM)),
                mint_msg,
            ),
            Err(ContractError::Unauthorized {})
        ));

        let mint_msg = ExecuteMsg::Mint {
            to: minter.clone(),
            token_id: token1.clone(),
            value: 1u64.into(),
            token_info: token_info1.clone(),
            msg: None,
        };
        // invalid mint, minter didn't specify fee
        assert!(matches!(
            execute(
                deps.as_mut(),
                mock_env(),
                mock_info(minter.as_ref(), &[]),
                mint_msg.clone(),
            ),
            Err(ContractError::Payment(PaymentError::NoFunds {}))
        ));
        // invalid mint, minter specify invalid amount
        assert!(matches!(
            execute(
                deps.as_mut(),
                mock_env(),
                mock_info(minter.as_ref(), &coins(3_000_000, NATIVE_DENOM)),
                mint_msg.clone(),
            ),
            Err(ContractError::Fee(FeeError::InsufficientFee(
                CREATION_FEE,
                3_000_000,
            )))
        ));

        // valid mint 1 token without royalty
        assert_eq!(
            execute(
                deps.as_mut(),
                mock_env(),
                mock_info(minter.as_ref(), &coins(CREATION_FEE, NATIVE_DENOM)),
                mint_msg,
            )
            .unwrap(),
            Response::new()
                .add_attribute("action", "transfer")
                .add_attribute("token_id", &token1)
                .add_attribute("amount", 1u64.to_string())
                .add_attribute("to", &minter)
        );
        // query total balance of token1
        assert_eq!(
            query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::Balance {
                    owner: minter.clone(),
                    token_id: token1.clone(),
                }
            ),
            to_binary(&BalanceResponse {
                balance: 1u64.into()
            })
        );
        // Query token info
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::TokenInfo { token_id: token1 },
        )
        .unwrap();
        let value: TokenInfoResponse = from_binary(&res).unwrap();
        assert_eq!(
            TokenInfoResponse {
                document_uri: token_info1.document_uri,
                image_uri: token_info1.image_uri,
                token_uri: token_info1.token_uri,
                royalty_info: None,
            },
            value
        );

        let mint_msg = ExecuteMsg::Mint {
            to: minter.clone(),
            token_id: token2.clone(),
            value: 5u64.into(),
            token_info: token_info2.clone(),
            msg: None,
        };
        // valid mint 5 tokens with royalty
        assert_eq!(
            execute(
                deps.as_mut(),
                mock_env(),
                mock_info(minter.as_ref(), &coins(CREATION_FEE, NATIVE_DENOM)),
                mint_msg.clone(),
            )
            .unwrap(),
            Response::new()
                .add_attribute("action", "transfer")
                .add_attribute("token_id", &token2)
                .add_attribute("amount", 5u64.to_string())
                .add_attribute("to", &minter)
        );

        // Query token info
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::TokenInfo {
                token_id: token2.clone(),
            },
        )
        .unwrap();
        let value: TokenInfoResponse = from_binary(&res).unwrap();
        let royalty_info = token_info2.royalty_info.unwrap();
        assert_eq!(
            TokenInfoResponse {
                document_uri: token_info2.document_uri,
                image_uri: token_info2.image_uri,
                token_uri: token_info2.token_uri,
                royalty_info: Some(RoyaltyInfoMsg {
                    payment_address: royalty_info.payment_address,
                    share: royalty_info.share
                }),
            },
            value
        );

        // valid mint another 5 tokens with royalty
        assert_eq!(
            execute(
                deps.as_mut(),
                mock_env(),
                mock_info(minter.as_ref(), &coins(CREATION_FEE, NATIVE_DENOM)),
                mint_msg,
            )
            .unwrap(),
            Response::new()
                .add_attribute("action", "transfer")
                .add_attribute("token_id", &token2)
                .add_attribute("amount", 5u64.to_string())
                .add_attribute("to", &minter)
        );
        // Query total balance of token2
        assert_eq!(
            query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::Balance {
                    owner: minter,
                    token_id: token2,
                }
            ),
            to_binary(&BalanceResponse {
                balance: 10u64.into() // 10 tokens in total
            })
        );
    }
}
