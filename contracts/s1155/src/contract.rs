use crate::error::ContractError;
use crate::event::{Event, TransferEvent};
use crate::msg::{BatchReceiveMsg, ExecuteMsg, QueryMsg, ReceiveMsg, TokenInfoResponse};
use crate::state::{TokenInfo, APPROVES, BALANCES, MINTER, TOKENS};
use cosmwasm_std::{entry_point, to_binary, Addr, Binary, Deps, Uint128};
use cosmwasm_std::{DepsMut, Env, MessageInfo, StdResult};
use cw1155::{Cw1155ExecuteMsg, Cw1155QueryMsg, TokenId};
use cw1155_base::contract::{execute as base_execute, query as base_query};
use cw1155_base::{msg::InstantiateMsg, ContractError as BaseError};
use cw2::set_contract_version;
use s1::{check_royalty_payment, MIN_ROYALTY_FEE};
use s2::{check_mint_payment, MIN_MINT_FEE};
use s_std::{error::FeeError, Response, SubMsg, NATIVE_DENOM};

// Version info for migration info
const CONTRACT_NAME: &str = "crates.io:s1155";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// TODO Change this
const MULTISIG: &str = "sign1nfvgxep88xrqza3534e92tlpnvvxctf4laa3kd";

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

/********************************* MESSAGES ***********************************/

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let env = ExecuteEnv { deps, env, info };
    match msg {
        ExecuteMsg::SendFrom {
            from,
            to,
            token_id,
            value,
            msg,
        } => execute_send_from(env, from, to, token_id, value, msg),
        ExecuteMsg::BatchSendFrom {
            from,
            to,
            batch,
            msg,
        } => execute_batch_send_from(env, from, to, batch, msg),
        ExecuteMsg::Mint {
            to,
            token_id,
            value,
            fee,
            token_info,
            msg,
        } => execute_mint(env, to, token_id, value, fee, token_info, msg),
        ExecuteMsg::BatchMint {
            to,
            total_fee,
            batch,
            token_info_batch,
            msg,
        } => execute_batch_mint(env, to, total_fee, batch, token_info_batch, msg),
        _ => {
            let result = base_execute(env.deps, env.env, env.info, Cw1155ExecuteMsg::from(msg));
            match result {
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
                },
            }
        }
    }
}

pub fn execute_send_from(
    env: ExecuteEnv,
    from: String,
    to: String,
    token_id: TokenId,
    amount: Uint128,
    msg: Option<Binary>,
) -> Result<Response, ContractError> {
    let ExecuteEnv {
        mut deps,
        env,
        info,
    } = env;

    let from_addr = deps.api.addr_validate(&from)?;
    let to_addr = deps.api.addr_validate(&to)?;

    let mut msgs = check_royalty_payment(&info, MIN_ROYALTY_FEE, MINTER.load(deps.storage)?)?;

    guard_can_approve(deps.as_ref(), &env, &from_addr, &info.sender)?;

    let mut rsp = Response::default();

    let event = execute_transfer_inner(
        &mut deps,
        Some(&from_addr),
        Some(&to_addr),
        &token_id,
        amount,
    )?;
    event.add_attributes(&mut rsp);

    if let Some(msg) = msg {
        msgs.push(SubMsg::new(
            ReceiveMsg {
                operator: info.sender.to_string(),
                from: Some(from),
                amount,
                token_id: token_id.clone(),
                msg,
            }
            .into_cosmos_msg(to)?,
        ));
    }
    rsp.messages = msgs;

    Ok(rsp)
}

pub fn execute_batch_send_from(
    env: ExecuteEnv,
    from: String,
    to: String,
    batch: Vec<(TokenId, Uint128)>,
    msg: Option<Binary>,
) -> Result<Response, ContractError> {
    let ExecuteEnv {
        mut deps,
        env,
        info,
    } = env;

    let from_addr = deps.api.addr_validate(&from)?;
    let to_addr = deps.api.addr_validate(&to)?;

    // MIN_ROYALTY_FEE * Number of Tokens
    let min_fee = Uint128::from(u128::try_from(batch.len()).unwrap())
        .checked_mul(Uint128::from(MIN_ROYALTY_FEE))
        .unwrap();

    let mut msgs = check_royalty_payment(&info, min_fee.u128(), MINTER.load(deps.storage)?)?;

    guard_can_approve(deps.as_ref(), &env, &from_addr, &info.sender)?;

    let mut rsp = Response::default();
    for (token_id, amount) in batch.iter() {
        let event = execute_transfer_inner(
            &mut deps,
            Some(&from_addr),
            Some(&to_addr),
            token_id,
            *amount,
        )?;
        event.add_attributes(&mut rsp);
    }

    if let Some(msg) = msg {
        msgs.push(SubMsg::new(
            BatchReceiveMsg {
                operator: info.sender.to_string(),
                from: Some(from),
                batch,
                msg,
            }
            .into_cosmos_msg(to)?,
        ));
    };
    rsp.messages = msgs;

    Ok(rsp)
}

pub fn execute_mint(
    env: ExecuteEnv,
    to: String,
    token_id: TokenId,
    amount: Uint128,
    fee: Uint128,
    token_info: TokenInfo,
    msg: Option<Binary>,
) -> Result<Response, ContractError> {
    let ExecuteEnv { mut deps, info, .. } = env;

    let multisig = deps.api.addr_validate(MULTISIG)?;
    let mut msgs = check_mint_payment(&info, fee.u128(), multisig)?;

    let to_addr = deps.api.addr_validate(&to)?;

    if info.sender != MINTER.load(deps.storage)? {
        return Err(ContractError::Unauthorized {});
    }

    let mut rsp = Response::default();

    let TokenInfo {
        document_uri,
        token_uri,
    } = token_info;

    let event = execute_transfer_inner(&mut deps, None, Some(&to_addr), &token_id, amount)?;
    event.add_attributes(&mut rsp);

    if let Some(msg) = msg {
        msgs.push(SubMsg::new(
            ReceiveMsg {
                operator: info.sender.to_string(),
                from: None,
                amount,
                token_id: token_id.clone(),
                msg,
            }
            .into_cosmos_msg(to)?,
        ));
    }
    rsp.messages = msgs;

    // insert if not exist
    if !TOKENS.has(deps.storage, &token_id) {
        // we must save some valid data here
        TOKENS.save(
            deps.storage,
            &token_id,
            &TokenInfo {
                document_uri,
                token_uri,
            },
        )?;
    }

    Ok(rsp)
}

pub fn execute_batch_mint(
    env: ExecuteEnv,
    to: String,
    total_fee: Uint128,
    batch: Vec<(TokenId, Uint128)>,
    token_info_batch: Vec<TokenInfo>,
    msg: Option<Binary>,
) -> Result<Response, ContractError> {
    let ExecuteEnv { mut deps, info, .. } = env;
    if info.sender != MINTER.load(deps.storage)? {
        return Err(ContractError::Unauthorized {});
    }

    if batch.len() != token_info_batch.len() {
        return Err(ContractError::TokenInfoMismatch {});
    }

    // MIN_MINT_FEE * Number of Tokens
    let min_fee = Uint128::from(u128::try_from(batch.len()).unwrap())
        .checked_mul(Uint128::from(MIN_MINT_FEE))
        .unwrap();

    if total_fee.u128() < min_fee.u128() {
        return Err(ContractError::Fee(FeeError::BelowMinFee(
            min_fee.u128(),
            NATIVE_DENOM.to_string(),
        )));
    }

    let multisig = deps.api.addr_validate(MULTISIG)?;
    let mut msgs = check_mint_payment(&info, min_fee.u128(), multisig)?;

    let to_addr = deps.api.addr_validate(&to)?;

    let mut rsp = Response::default();

    for (pos, (token_id, amount)) in batch.iter().enumerate() {
        let event = execute_transfer_inner(&mut deps, None, Some(&to_addr), token_id, *amount)?;
        event.add_attributes(&mut rsp);

        let token_info = token_info_batch
            .get(pos)
            .ok_or(ContractError::TokenInfoMismatch {})?;

        let TokenInfo {
            document_uri,
            token_uri,
        } = token_info;

        // insert if not exist
        if !TOKENS.has(deps.storage, token_id) {
            // we must save some valid data here
            TOKENS.save(
                deps.storage,
                token_id,
                &TokenInfo {
                    document_uri: document_uri.to_string(),
                    token_uri: token_uri.to_string(),
                },
            )?;
        }
    }

    if let Some(msg) = msg {
        msgs.push(SubMsg::new(
            BatchReceiveMsg {
                operator: info.sender.to_string(),
                from: None,
                batch,
                msg,
            }
            .into_cosmos_msg(to)?,
        ))
    };
    rsp.messages = msgs;

    Ok(rsp)
}

/********************************* QUERIES ************************************/

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::TokenInfo { token_id } => to_binary(&query_config(deps, token_id)?),
        _ => base_query(deps, env, Cw1155QueryMsg::from(msg)),
    }
}

fn query_config(deps: Deps, token_id: TokenId) -> StdResult<TokenInfoResponse> {
    let TokenInfo {
        document_uri,
        token_uri,
    } = TOKENS.load(deps.storage, &token_id)?;
    Ok(TokenInfoResponse {
        document_uri,
        token_uri,
    })
}

/********************************* HELPERS ************************************/

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

/// returns true iff the sender can execute approve or reject on the contract
fn check_can_approve(deps: Deps, env: &Env, owner: &Addr, operator: &Addr) -> StdResult<bool> {
    // owner can approve
    if owner == operator {
        return Ok(true);
    }
    // operator can approve
    let op = APPROVES.may_load(deps.storage, (owner, operator))?;
    Ok(match op {
        Some(ex) => !ex.is_expired(&env.block),
        None => false,
    })
}

fn guard_can_approve(
    deps: Deps,
    env: &Env,
    owner: &Addr,
    operator: &Addr,
) -> Result<(), ContractError> {
    if !check_can_approve(deps, env, owner, operator)? {
        Err(ContractError::Unauthorized {})
    } else {
        Ok(())
    }
}

/********************************* TESTS ************************************/

#[cfg(test)]
mod tests {
    use cosmwasm_std::{
        coins, from_binary,
        testing::{mock_dependencies, mock_env, mock_info},
        to_binary, BankMsg,
    };
    use cw1155::{BalanceResponse, BatchBalanceResponse};
    use s_std::create_fund_community_pool_msg;

    use super::*;

    #[test]
    fn test_send() {
        let minter = String::from("minter");
        let user1 = String::from("user1");
        let user2 = String::from("user2");

        let token1 = "token1".to_owned();
        let token_info1 = TokenInfo {
            document_uri: "https://example.com/document1".to_string(),
            token_uri: "https://example.com/token_uri1".to_string(),
        };
        let mut deps = mock_dependencies();

        // instantiate contract for "minter"
        let msg = InstantiateMsg {
            minter: minter.clone(),
        };
        let res = instantiate(deps.as_mut(), mock_env(), mock_info("operator", &[]), msg).unwrap();
        assert_eq!(0, res.messages.len());

        // Mint token
        let mint_msg = ExecuteMsg::Mint {
            to: minter.clone(),
            token_id: token1.clone(),
            fee: Uint128::from(MIN_MINT_FEE),
            value: 2u64.into(),
            token_info: token_info1,
            msg: None,
        };
        execute(
            deps.as_mut(),
            mock_env(),
            mock_info(minter.as_ref(), &coins(MIN_MINT_FEE, NATIVE_DENOM)),
            mint_msg,
        )
        .unwrap();

        let transfer_msg = ExecuteMsg::SendFrom {
            from: minter.clone(),
            to: user1.clone(),
            token_id: token1.clone(),
            value: 1u64.into(),
            msg: None,
        };

        // Minter SIGN balance below royalty fee
        assert!(matches!(
            execute(
                deps.as_mut(),
                mock_env(),
                mock_info(minter.as_ref(), &coins(500u128, NATIVE_DENOM)),
                transfer_msg.clone(),
            ),
            Err(ContractError::Fee(FeeError::InsufficientFee(
                MIN_ROYALTY_FEE,
                500u128
            )))
        ));
        // user1 not approved to transfer minter's token
        assert!(matches!(
            execute(
                deps.as_mut(),
                mock_env(),
                mock_info(user1.as_ref(), &coins(MIN_ROYALTY_FEE, NATIVE_DENOM)),
                transfer_msg.clone(),
            ),
            Err(ContractError::Unauthorized {})
        ));

        // Valid transfer
        let bank_msg = SubMsg::new(BankMsg::Send {
            to_address: minter.to_string(),
            amount: coins(950u128, NATIVE_DENOM.to_string()),
        });
        let community_msg =
            SubMsg::new(create_fund_community_pool_msg(coins(50u128, NATIVE_DENOM)));
        let mut rsp = Response::new()
            .add_attribute("action", "transfer")
            .add_attribute("token_id", &token1)
            .add_attribute("amount", 1u64.to_string())
            .add_attribute("from", &minter)
            .add_attribute("to", &user1);
        rsp.messages = vec![bank_msg, community_msg];
        assert_eq!(
            execute(
                deps.as_mut(),
                mock_env(),
                mock_info(minter.as_ref(), &coins(MIN_ROYALTY_FEE, NATIVE_DENOM)),
                transfer_msg,
            )
            .unwrap(),
            rsp
        );

        // query balance of token1 belonging to minter
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

        // query balance of token1 belonging to user1
        assert_eq!(
            query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::Balance {
                    owner: user1,
                    token_id: token1.clone(),
                }
            ),
            to_binary(&BalanceResponse {
                balance: 1u64.into()
            })
        );

        // approve user2 to perform transfer for minter
        execute(
            deps.as_mut(),
            mock_env(),
            mock_info(minter.as_ref(), &[]),
            ExecuteMsg::ApproveAll {
                operator: user2.clone(),
                expires: None,
            },
        )
        .unwrap();

        // Valid transfer from minter to user2 using user2 account
        let transfer_msg = ExecuteMsg::SendFrom {
            from: minter.clone(),
            to: user2.clone(),
            token_id: token1.clone(),
            value: 1u64.into(),
            msg: None,
        };
        let bank_msg = SubMsg::new(BankMsg::Send {
            to_address: minter.to_string(),
            amount: coins(950u128, NATIVE_DENOM.to_string()),
        });
        let community_msg =
            SubMsg::new(create_fund_community_pool_msg(coins(50u128, NATIVE_DENOM)));
        let mut rsp = Response::new()
            .add_attribute("action", "transfer")
            .add_attribute("token_id", &token1)
            .add_attribute("amount", 1u64.to_string())
            .add_attribute("from", &minter)
            .add_attribute("to", &user2);
        rsp.messages = vec![bank_msg, community_msg];
        assert_eq!(
            execute(
                deps.as_mut(),
                mock_env(),
                mock_info(user2.as_ref(), &coins(MIN_ROYALTY_FEE, NATIVE_DENOM)),
                transfer_msg,
            )
            .unwrap(),
            rsp
        );

        // query balance of token1 belonging to minter
        assert_eq!(
            query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::Balance {
                    owner: minter,
                    token_id: token1.clone(),
                }
            ),
            to_binary(&BalanceResponse {
                balance: 0u64.into()
            })
        );

        // query balance of token1 belonging to user2
        assert_eq!(
            query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::Balance {
                    owner: user2,
                    token_id: token1,
                }
            ),
            to_binary(&BalanceResponse {
                balance: 1u64.into()
            })
        );
    }

    #[test]
    fn test_send_all() {
        let minter = String::from("minter");
        let user1 = String::from("user1");
        let user2 = String::from("user2");

        let token1 = "token1".to_owned();
        let token_info1 = TokenInfo {
            document_uri: "https://example.com/document1".to_string(),
            token_uri: "https://example.com/token_uri1".to_string(),
        };
        let token2 = "token2".to_owned();
        let token_info2 = TokenInfo {
            document_uri: "https://example.com/document2".to_string(),
            token_uri: "https://example.com/token_uri2".to_string(),
        };
        let payment = MIN_ROYALTY_FEE * 2; // Min royalty fee 2 tokens
        let mut deps = mock_dependencies();

        // instantiate contract for "minter"
        let msg = InstantiateMsg {
            minter: minter.clone(),
        };
        let res = instantiate(deps.as_mut(), mock_env(), mock_info("operator", &[]), msg).unwrap();
        assert_eq!(0, res.messages.len());

        // mint tokens
        let mint_msg = ExecuteMsg::BatchMint {
            to: minter.clone(),
            total_fee: Uint128::from(MIN_MINT_FEE * 2),
            batch: vec![
                (token1.clone(), Uint128::from(1u128)),
                (token2.clone(), Uint128::from(3u128)),
            ],
            token_info_batch: vec![token_info1, token_info2],
            msg: None,
        };
        execute(
            deps.as_mut(),
            mock_env(),
            mock_info(minter.as_ref(), &coins(MIN_MINT_FEE * 2, NATIVE_DENOM)),
            mint_msg,
        )
        .unwrap();

        let transfer_msg = ExecuteMsg::BatchSendFrom {
            from: minter.clone(),
            to: user1.clone(),
            batch: vec![
                (token1.clone(), Uint128::from(1u128)),
                (token2.clone(), Uint128::from(2u128)),
            ],
            msg: None,
        };

        // Minter SIGN balance below total royalty fee needed
        assert!(matches!(
            execute(
                deps.as_mut(),
                mock_env(),
                mock_info(minter.as_ref(), &coins(MIN_ROYALTY_FEE, NATIVE_DENOM)),
                transfer_msg.clone(),
            ),
            Err(ContractError::Fee(FeeError::InsufficientFee(
                _payment,
                MIN_ROYALTY_FEE
            )))
        ));

        // user1 not approved to transfer minter's tokens
        assert!(matches!(
            execute(
                deps.as_mut(),
                mock_env(),
                mock_info(user1.as_ref(), &coins(payment, NATIVE_DENOM)),
                transfer_msg.clone(),
            ),
            Err(ContractError::Unauthorized {})
        ));

        // Valid transfer
        let bank_msg = SubMsg::new(BankMsg::Send {
            to_address: minter.to_string(),
            amount: coins(1900u128, NATIVE_DENOM.to_string()),
        });
        let community_msg =
            SubMsg::new(create_fund_community_pool_msg(coins(100u128, NATIVE_DENOM)));
        let mut rsp = Response::new()
            .add_attribute("action", "transfer")
            .add_attribute("token_id", &token1)
            .add_attribute("amount", 1u64.to_string())
            .add_attribute("from", &minter)
            .add_attribute("to", &user1)
            .add_attribute("action", "transfer")
            .add_attribute("token_id", &token2)
            .add_attribute("amount", 2u64.to_string())
            .add_attribute("from", &minter)
            .add_attribute("to", &user1);
        rsp.messages = vec![bank_msg, community_msg];
        assert_eq!(
            execute(
                deps.as_mut(),
                mock_env(),
                mock_info(minter.as_ref(), &coins(payment, NATIVE_DENOM)),
                transfer_msg,
            )
            .unwrap(),
            rsp
        );

        // query minter total balance of token1 and token2
        assert_eq!(
            query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::BatchBalance {
                    owner: minter.clone(),
                    token_ids: vec![token1.clone(), token2.clone()],
                }
            ),
            to_binary(&BatchBalanceResponse {
                balances: vec![Uint128::from(0u128), Uint128::from(1u128)]
            })
        );

        // query user1 total balance of token1 and token2
        assert_eq!(
            query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::BatchBalance {
                    owner: user1.clone(),
                    token_ids: vec![token1.clone(), token2.clone()],
                }
            ),
            to_binary(&BatchBalanceResponse {
                balances: vec![Uint128::from(1u128), Uint128::from(2u128)]
            })
        );

        // approve user2 to perform transfer for user1
        execute(
            deps.as_mut(),
            mock_env(),
            mock_info(user1.as_ref(), &[]),
            ExecuteMsg::ApproveAll {
                operator: user2.clone(),
                expires: None,
            },
        )
        .unwrap();

        // valid transfer from user1 to minter using user2 account
        let transfer_msg = ExecuteMsg::BatchSendFrom {
            from: user1.clone(),
            to: minter.clone(),
            batch: vec![
                (token1.clone(), Uint128::from(1u128)),
                (token2.clone(), Uint128::from(1u128)),
            ],
            msg: None,
        };
        let bank_msg = SubMsg::new(BankMsg::Send {
            to_address: minter.to_string(),
            amount: coins(1900u128, NATIVE_DENOM.to_string()),
        });
        let community_msg =
            SubMsg::new(create_fund_community_pool_msg(coins(100u128, NATIVE_DENOM)));
        let mut rsp = Response::new()
            .add_attribute("action", "transfer")
            .add_attribute("token_id", &token1)
            .add_attribute("amount", 1u64.to_string())
            .add_attribute("from", &user1)
            .add_attribute("to", &minter)
            .add_attribute("action", "transfer")
            .add_attribute("token_id", &token2)
            .add_attribute("amount", 1u64.to_string())
            .add_attribute("from", &user1)
            .add_attribute("to", &minter);
        rsp.messages = vec![bank_msg, community_msg];
        assert_eq!(
            execute(
                deps.as_mut(),
                mock_env(),
                mock_info(user2.as_ref(), &coins(payment, NATIVE_DENOM)),
                transfer_msg,
            )
            .unwrap(),
            rsp
        );

        // query minter total balance of token1 and token2
        assert_eq!(
            query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::BatchBalance {
                    owner: minter,
                    token_ids: vec![token1.clone(), token2.clone()],
                }
            ),
            to_binary(&BatchBalanceResponse {
                balances: vec![Uint128::from(1u128), Uint128::from(2u128)]
            })
        );

        // query user1 total balance of token1 and token2
        assert_eq!(
            query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::BatchBalance {
                    owner: user1,
                    token_ids: vec![token1, token2],
                }
            ),
            to_binary(&BatchBalanceResponse {
                balances: vec![Uint128::from(0u128), Uint128::from(1u128)]
            })
        );
    }

    #[test]
    fn test_mint() {
        let minter = String::from("minter");
        let user1 = String::from("user1");

        let token1 = "token1".to_owned();
        let token_info1 = TokenInfo {
            document_uri: "https://example.com/document1".to_string(),
            token_uri: "https://example.com/token_uri1".to_string(),
        };

        let mut deps = mock_dependencies();
        // instantiate contract for "minter"
        let msg = InstantiateMsg {
            minter: minter.clone(),
        };
        let res = instantiate(deps.as_mut(), mock_env(), mock_info("operator", &[]), msg).unwrap();
        assert_eq!(0, res.messages.len());

        let mint_msg = ExecuteMsg::Mint {
            to: minter.clone(),
            token_id: token1.clone(),
            fee: Uint128::from(MIN_MINT_FEE),
            value: 1u64.into(),
            token_info: token_info1.clone(),
            msg: None,
        };

        // invalid mint, user1 don't mint permission on "minter" contract
        assert!(matches!(
            execute(
                deps.as_mut(),
                mock_env(),
                mock_info(user1.as_ref(), &coins(MIN_MINT_FEE, NATIVE_DENOM)),
                mint_msg,
            ),
            Err(ContractError::Unauthorized {})
        ));

        let mint_msg = ExecuteMsg::Mint {
            to: minter.clone(),
            token_id: token1.clone(),
            fee: Uint128::from(15_000_000u128),
            value: 1u64.into(),
            token_info: token_info1.clone(),
            msg: None,
        };

        // invalid mint, minter specify below min fee
        let _demon_string = NATIVE_DENOM.to_string();
        assert!(matches!(
            execute(
                deps.as_mut(),
                mock_env(),
                mock_info(minter.as_ref(), &coins(MIN_MINT_FEE, NATIVE_DENOM)),
                mint_msg,
            ),
            Err(ContractError::Fee(FeeError::BelowMinFee(
                MIN_MINT_FEE,
                _demon_string
            )))
        ));

        let mint_msg = ExecuteMsg::Mint {
            to: minter.clone(),
            token_id: token1.clone(),
            fee: Uint128::from(MIN_MINT_FEE),
            value: 1u64.into(),
            token_info: token_info1.clone(),
            msg: None,
        };

        // invalid mint, minter don't have enough funds
        assert!(matches!(
            execute(
                deps.as_mut(),
                mock_env(),
                mock_info(minter.as_ref(), &coins(15_000_000, NATIVE_DENOM)),
                mint_msg.clone(),
            ),
            Err(ContractError::Fee(FeeError::InsufficientFee(
                MIN_MINT_FEE,
                15_000_000
            )))
        ));

        // valid mint 1 token without royalty
        let bank_msg = SubMsg::new(BankMsg::Send {
            to_address: MULTISIG.to_string(),
            amount: coins(MIN_MINT_FEE, NATIVE_DENOM.to_string()),
        });
        let mut rsp = Response::new()
            .add_attribute("action", "transfer")
            .add_attribute("token_id", &token1)
            .add_attribute("amount", 1u64.to_string())
            .add_attribute("to", &minter);
        rsp.messages = vec![bank_msg];
        assert_eq!(
            execute(
                deps.as_mut(),
                mock_env(),
                mock_info(minter.as_ref(), &coins(MIN_MINT_FEE, NATIVE_DENOM)),
                mint_msg,
            )
            .unwrap(),
            rsp
        );

        // query total balance of token1
        assert_eq!(
            query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::Balance {
                    owner: minter,
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
                token_uri: token_info1.token_uri,
            },
            value
        );
    }

    #[test]
    fn test_batch_mint() {
        let minter = String::from("minter");
        let user1 = String::from("user1");

        let token1 = "token1".to_owned();
        let token_info1 = TokenInfo {
            document_uri: "https://example.com/document1".to_string(),
            token_uri: "https://example.com/token_uri1".to_string(),
        };
        let token2 = "token2".to_owned();
        let token_info2 = TokenInfo {
            document_uri: "https://example.com/document2".to_string(),
            token_uri: "https://example.com/token_uri2".to_string(),
        };

        let token_batch = vec![
            (token1.clone(), Uint128::from(1u128)),
            (token2.clone(), Uint128::from(3u128)),
        ];
        let token_info_batch_half = vec![token_info1.clone()];
        let token_info_batch = vec![token_info1.clone(), token_info2.clone()];
        let payment = MIN_MINT_FEE * 2; // Min amount to be paid
        let demon_string = NATIVE_DENOM.to_string();

        let mut deps = mock_dependencies();
        // instantiate contract for "minter"
        let msg = InstantiateMsg {
            minter: minter.clone(),
        };
        let res = instantiate(deps.as_mut(), mock_env(), mock_info("operator", &[]), msg).unwrap();
        assert_eq!(0, res.messages.len());

        let mint_msg = ExecuteMsg::BatchMint {
            to: minter.clone(),
            total_fee: Uint128::from(MIN_MINT_FEE),
            batch: token_batch.clone(),
            token_info_batch: token_info_batch_half,
            msg: None,
        };
        // Mismatch number of tokens and token info
        assert!(matches!(
            execute(
                deps.as_mut(),
                mock_env(),
                mock_info(minter.as_ref(), &coins(MIN_MINT_FEE, NATIVE_DENOM)),
                mint_msg,
            ),
            Err(ContractError::TokenInfoMismatch {})
        ));

        let mint_msg = ExecuteMsg::BatchMint {
            to: minter.clone(),
            total_fee: Uint128::from(30_000_000u128),
            batch: token_batch.clone(),
            token_info_batch: token_info_batch.clone(),
            msg: None,
        };
        // invalid mint, minter specify below min fee
        assert!(matches!(
            execute(
                deps.as_mut(),
                mock_env(),
                mock_info(minter.as_ref(), &coins(payment, NATIVE_DENOM)),
                mint_msg,
            ),
            Err(ContractError::Fee(FeeError::BelowMinFee(
                _payment,
                _demon_string
            )))
        ));

        let mint_msg = ExecuteMsg::BatchMint {
            to: minter.clone(),
            total_fee: Uint128::from(payment),
            batch: token_batch,
            token_info_batch,
            msg: None,
        };
        // invalid mint, user1 don't mint permission on "minter" contract
        assert!(matches!(
            execute(
                deps.as_mut(),
                mock_env(),
                mock_info(user1.as_ref(), &coins(payment, NATIVE_DENOM)),
                mint_msg.clone(),
            ),
            Err(ContractError::Unauthorized {})
        ));
        // invalid mint, minter don't have enough funds
        assert!(matches!(
            execute(
                deps.as_mut(),
                mock_env(),
                mock_info(minter.as_ref(), &coins(15_000_000u128, NATIVE_DENOM)),
                mint_msg.clone(),
            ),
            Err(ContractError::Fee(FeeError::InsufficientFee(
                _payment,
                15_000_000u128
            )))
        ));

        // valid mint 2 different token
        let bank_msg = SubMsg::new(BankMsg::Send {
            to_address: MULTISIG.to_string(),
            amount: coins(payment, demon_string),
        });
        let mut rsp = Response::new()
            .add_attribute("action", "transfer")
            .add_attribute("token_id", &token1)
            .add_attribute("amount", 1u64.to_string())
            .add_attribute("to", &minter)
            .add_attribute("action", "transfer")
            .add_attribute("token_id", &token2)
            .add_attribute("amount", 3u64.to_string())
            .add_attribute("to", &minter);
        rsp.messages = vec![bank_msg];
        assert_eq!(
            execute(
                deps.as_mut(),
                mock_env(),
                mock_info(minter.as_ref(), &coins(payment, NATIVE_DENOM)),
                mint_msg,
            )
            .unwrap(),
            rsp
        );

        // query total balance of token1 and token2
        assert_eq!(
            query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::BatchBalance {
                    owner: minter,
                    token_ids: vec![token1.clone(), token2.clone()],
                }
            ),
            to_binary(&BatchBalanceResponse {
                balances: vec![Uint128::from(1u128), Uint128::from(3u128)]
            })
        );

        // Query token1 info
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
                token_uri: token_info1.token_uri,
            },
            value
        );

        // Query token2 info
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::TokenInfo { token_id: token2 },
        )
        .unwrap();
        let value: TokenInfoResponse = from_binary(&res).unwrap();
        assert_eq!(
            TokenInfoResponse {
                document_uri: token_info2.document_uri,
                token_uri: token_info2.token_uri,
            },
            value
        );
    }
}
