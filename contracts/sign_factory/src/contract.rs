use cosmwasm_std::{entry_point, to_binary, Binary, Deps};
use cosmwasm_std::{DepsMut, Empty, Env, MessageInfo, StdResult};

use cw2::set_contract_version;

use crate::msg::{InstantiateMsg, QueryMsg, S1155Response, S721Response};
use crate::state::{S1155_STORE, S721_STORE};
use crate::ContractError;
use s_std::{FactoryExecuteMsg as ExecuteMsg, Response};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:sign_factory";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut<Empty>,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::new()
        .add_attribute("contract_name", CONTRACT_NAME)
        .add_attribute("contract_version", CONTRACT_VERSION))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut<Empty>,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::AddS1155 { from } => execute_add_s1155(deps, env, info, from),
        ExecuteMsg::AddS721 { from } => execute_add_s721(deps, env, info, from),
    }
}

pub fn execute_add_s1155(
    deps: DepsMut<Empty>,
    _env: Env,
    info: MessageInfo,
    from: String,
) -> Result<Response, ContractError> {
    let sender = deps.api.addr_validate(from.as_str())?;

    if S1155_STORE.has(deps.storage, &sender) {
        return Err(ContractError::OneS1155 {});
    }

    S1155_STORE.save(deps.storage, &sender, &info.sender)?;

    Ok(Response::new()
        .add_attribute("sender", from)
        .add_attribute("contract_addr", info.sender.as_str()))
}

pub fn execute_add_s721(
    deps: DepsMut<Empty>,
    _env: Env,
    info: MessageInfo,
    from: String,
) -> Result<Response, ContractError> {
    let sender = deps.api.addr_validate(from.as_str())?;

    let mut store = S721_STORE
        .may_load(deps.storage, &sender)?
        .unwrap_or_default();

    if store.contains(&info.sender) {
        return Err(ContractError::AlreadyExist {
            contract_addr: info.sender.to_string(),
        });
    }

    store.push(info.sender.clone());

    S721_STORE.save(deps.storage, &sender, &store)?;

    Ok(Response::new()
        .add_attribute("sender", sender)
        .add_attribute("contract_addr", info.sender.as_str()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::S1155 { from } => query_s1155(deps, env, from),
        QueryMsg::S721 { from } => query_s721(deps, env, from),
    }
}

pub fn query_s1155(deps: Deps, _env: Env, from: String) -> StdResult<Binary> {
    let res = S1155_STORE.may_load(deps.storage, &deps.api.addr_validate(from.as_str())?)?;
    match res {
        Some(addr) => to_binary(&S1155Response {
            contract_addr: Some(addr.to_string()),
        }),
        None => to_binary(&S1155Response {
            contract_addr: None,
        }),
    }
}

pub fn query_s721(deps: Deps, _env: Env, from: String) -> StdResult<Binary> {
    let res = &S721_STORE.may_load(deps.storage, &deps.api.addr_validate(from.as_str())?)?;
    match res {
        Some(addrs) => {
            let humanize_addrs = addrs
                .iter()
                .map(|f| Ok(f.to_string()))
                .collect::<StdResult<Vec<String>>>()?;
            to_binary(&S721Response {
                contract_addrs: humanize_addrs,
            })
        }
        None => to_binary(&S721Response {
            contract_addrs: vec![],
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};

    #[test]
    fn proper_initializations() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {};
        let info = mock_info("creator", &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(
            Response::new()
                .add_attribute("contract_name", CONTRACT_NAME)
                .add_attribute("contract_version", CONTRACT_VERSION),
            res
        )
    }

    #[test]
    fn test_execute_add_s1155() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {};
        let info = mock_info("creator", &[]);
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let contract1 = mock_info("contract1", &[]);
        let contract2 = mock_info("contract2", &[]);

        // Add contract to user1
        let msg = ExecuteMsg::AddS1155 {
            from: "user1".to_string(),
        };
        let res = execute(deps.as_mut(), mock_env(), contract1, msg).unwrap();
        assert_eq!(
            Response::new()
                .add_attribute("sender", "user1")
                .add_attribute("contract_addr", "contract1"),
            res
        );

        // Error: Unable to add another contract for user1
        let msg = ExecuteMsg::AddS1155 {
            from: "user1".to_string(),
        };
        assert!(matches!(
            execute(deps.as_mut(), mock_env(), contract2.clone(), msg),
            Err(ContractError::OneS1155 {})
        ));

        // Add contract to user2
        let msg = ExecuteMsg::AddS1155 {
            from: "user2".to_string(),
        };
        let res = execute(deps.as_mut(), mock_env(), contract2, msg).unwrap();
        assert_eq!(
            Response::new()
                .add_attribute("sender", "user2")
                .add_attribute("contract_addr", "contract2"),
            res
        );
    }

    #[test]
    fn test_execute_add_s721() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {};
        let info = mock_info("creator", &[]);
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let contract1 = mock_info("contract1", &[]);
        let contract2 = mock_info("contract2", &[]);
        let contract3 = mock_info("contract3", &[]);
        let contract4 = mock_info("contract4", &[]);

        // Add contract to user1
        let msg = ExecuteMsg::AddS721 {
            from: "user1".to_string(),
        };
        let res = execute(deps.as_mut(), mock_env(), contract1.clone(), msg.clone()).unwrap();
        assert_eq!(
            Response::new()
                .add_attribute("sender", "user1")
                .add_attribute("contract_addr", "contract1"),
            res
        );

        // Error: Add existing contract to user1
        let _address = "contract1".to_string();
        assert!(matches!(
            execute(deps.as_mut(), mock_env(), contract1, msg),
            Err(ContractError::AlreadyExist {
                contract_addr: _address
            })
        ));

        // Add another contract to user1
        let msg = ExecuteMsg::AddS721 {
            from: "user1".to_string(),
        };
        let res = execute(deps.as_mut(), mock_env(), contract2, msg).unwrap();
        assert_eq!(
            Response::new()
                .add_attribute("sender", "user1")
                .add_attribute("contract_addr", "contract2"),
            res
        );

        // Add contract to user2
        let msg = ExecuteMsg::AddS721 {
            from: "user2".to_string(),
        };
        let res = execute(deps.as_mut(), mock_env(), contract3, msg).unwrap();
        assert_eq!(
            Response::new()
                .add_attribute("sender", "user2")
                .add_attribute("contract_addr", "contract3"),
            res
        );

        // Add another contract to user2
        let msg = ExecuteMsg::AddS721 {
            from: "user2".to_string(),
        };
        let res = execute(deps.as_mut(), mock_env(), contract4, msg).unwrap();
        assert_eq!(
            Response::new()
                .add_attribute("sender", "user2")
                .add_attribute("contract_addr", "contract4"),
            res
        );
    }

    #[test]
    fn test_query_s1155() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {};
        let info = mock_info("creator", &[]);
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let user1 = mock_info("user1", &[]);
        let user2 = mock_info("user2", &[]);

        let contract1 = mock_info("contract1", &[]);
        let contract2 = mock_info("contract2", &[]);

        // Query user1 s1155 address, Should be empty
        let msg = QueryMsg::S1155 {
            from: user1.sender.to_string(),
        };
        assert_eq!(
            to_binary(&S1155Response {
                contract_addr: None
            }),
            query(deps.as_ref(), mock_env(), msg)
        );

        // Add contract to user1
        let msg = ExecuteMsg::AddS1155 {
            from: user1.sender.to_string(),
        };
        execute(deps.as_mut(), mock_env(), contract1, msg).unwrap();

        // Query user1 s1155 address, Should have "addr0001"
        let msg = QueryMsg::S1155 {
            from: user1.sender.to_string(),
        };
        assert_eq!(
            to_binary(&S1155Response {
                contract_addr: Some("contract1".to_string())
            }),
            query(deps.as_ref(), mock_env(), msg)
        );

        // Query user2 s1155 address, Should be empty
        let msg = QueryMsg::S1155 {
            from: user2.sender.to_string(),
        };
        assert_eq!(
            to_binary(&S1155Response {
                contract_addr: None
            }),
            query(deps.as_ref(), mock_env(), msg)
        );

        // Add contract to user2
        let msg = ExecuteMsg::AddS1155 {
            from: user2.sender.to_string(),
        };
        execute(deps.as_mut(), mock_env(), contract2, msg).unwrap();

        // Query user2 s1155 address, Should have "addr0002"
        let msg = QueryMsg::S1155 {
            from: user2.sender.to_string(),
        };
        assert_eq!(
            to_binary(&S1155Response {
                contract_addr: Some("contract2".to_string())
            }),
            query(deps.as_ref(), mock_env(), msg)
        );
    }

    #[test]
    fn test_query_s721() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {};
        let info = mock_info("creator", &[]);
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let user1 = mock_info("user1", &[]);
        let user2 = mock_info("user2", &[]);

        let contract1 = mock_info("contract1", &[]);
        let contract2 = mock_info("contract2", &[]);

        // Query user1 s1155 address, Should be empty
        let msg = QueryMsg::S721 {
            from: user1.sender.to_string(),
        };
        assert_eq!(
            to_binary(&S721Response {
                contract_addrs: vec![]
            }),
            query(deps.as_ref(), mock_env(), msg)
        );

        // Add contract to user1
        let msg = ExecuteMsg::AddS721 {
            from: user1.sender.to_string(),
        };
        execute(deps.as_mut(), mock_env(), contract1, msg).unwrap();

        // Query user1 s1155 address, should have 1 address
        let msg = QueryMsg::S721 {
            from: user1.sender.to_string(),
        };
        assert_eq!(
            to_binary(&S721Response {
                contract_addrs: vec!["contract1".to_string()]
            }),
            query(deps.as_ref(), mock_env(), msg)
        );

        // Add another contract to user1
        let msg = ExecuteMsg::AddS721 {
            from: user1.sender.to_string(),
        };
        execute(deps.as_mut(), mock_env(), contract2, msg).unwrap();

        // Query user1 s1155 address, should have 2 address
        let msg = QueryMsg::S721 {
            from: user1.sender.to_string(),
        };
        assert_eq!(
            to_binary(&S721Response {
                contract_addrs: vec!["contract1".to_string(), "contract2".to_string()]
            }),
            query(deps.as_ref(), mock_env(), msg)
        );

        // Query user2 s1155 address, Should be empty
        let msg = QueryMsg::S721 {
            from: user2.sender.to_string(),
        };
        assert_eq!(
            to_binary(&S721Response {
                contract_addrs: vec![]
            }),
            query(deps.as_ref(), mock_env(), msg)
        );
    }
}
