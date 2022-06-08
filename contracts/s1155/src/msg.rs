use crate::state::TokenInfo;
use cosmwasm_std::{to_binary, Binary, StdResult, Uint128, WasmMsg};
use cw1155::{Cw1155ExecuteMsg, Cw1155QueryMsg, TokenId};
use cw_utils::Expiration;
use s_std::CosmosMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    SendFrom {
        from: String,
        to: String,
        token_id: TokenId,
        value: Uint128,
        msg: Option<Binary>,
    },
    BatchSendFrom {
        from: String,
        to: String,
        batch: Vec<(TokenId, Uint128)>,
        msg: Option<Binary>,
    },
    Mint {
        to: String,
        token_id: TokenId,
        value: Uint128,
        token_info: TokenInfo,
        msg: Option<Binary>,
    },
    BatchMint {
        to: String,
        batch: Vec<(TokenId, Uint128)>,
        token_info_batch: Vec<TokenInfo>,
        msg: Option<Binary>,
    },
    Burn {
        from: String,
        token_id: TokenId,
        value: Uint128,
    },
    BatchBurn {
        from: String,
        batch: Vec<(TokenId, Uint128)>,
    },
    ApproveAll {
        operator: String,
        expires: Option<Expiration>,
    },
    RevokeAll {
        operator: String,
    },
}

impl From<ExecuteMsg> for Cw1155ExecuteMsg {
    fn from(msg: ExecuteMsg) -> Cw1155ExecuteMsg {
        match msg {
            ExecuteMsg::SendFrom {
                from,
                to,
                token_id,
                value,
                msg,
            } => Cw1155ExecuteMsg::SendFrom {
                from,
                to,
                token_id,
                value,
                msg,
            },
            ExecuteMsg::BatchSendFrom {
                from,
                to,
                batch,
                msg,
            } => Cw1155ExecuteMsg::BatchSendFrom {
                from,
                to,
                batch,
                msg,
            },
            ExecuteMsg::Burn {
                from,
                token_id,
                value,
            } => Cw1155ExecuteMsg::Burn {
                from,
                token_id,
                value,
            },
            ExecuteMsg::BatchBurn { from, batch } => Cw1155ExecuteMsg::BatchBurn { from, batch },
            ExecuteMsg::ApproveAll { operator, expires } => {
                Cw1155ExecuteMsg::ApproveAll { operator, expires }
            }
            ExecuteMsg::RevokeAll { operator } => Cw1155ExecuteMsg::RevokeAll { operator },
            _ => unreachable!("cannot convert {:?} to Cw1155ExecuteMsg", msg),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Balance {
        owner: String,
        token_id: TokenId,
    },
    BatchBalance {
        owner: String,
        token_ids: Vec<TokenId>,
    },
    ApprovedForAll {
        owner: String,
        include_expired: Option<bool>,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    IsApprovedForAll {
        owner: String,
        operator: String,
    },
    TokenInfo {
        token_id: TokenId,
    },
    Tokens {
        owner: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    AllTokens {
        start_after: Option<String>,
        limit: Option<u32>,
    },
}

impl From<QueryMsg> for Cw1155QueryMsg {
    fn from(msg: QueryMsg) -> Cw1155QueryMsg {
        match msg {
            QueryMsg::Balance { owner, token_id } => Cw1155QueryMsg::Balance { owner, token_id },
            QueryMsg::BatchBalance { owner, token_ids } => {
                Cw1155QueryMsg::BatchBalance { owner, token_ids }
            }
            QueryMsg::ApprovedForAll {
                owner,
                include_expired,
                start_after,
                limit,
            } => Cw1155QueryMsg::ApprovedForAll {
                owner,
                include_expired,
                start_after,
                limit,
            },
            QueryMsg::Tokens {
                owner,
                start_after,
                limit,
            } => Cw1155QueryMsg::Tokens {
                owner,
                start_after,
                limit,
            },
            QueryMsg::AllTokens { start_after, limit } => {
                Cw1155QueryMsg::AllTokens { start_after, limit }
            }
            QueryMsg::IsApprovedForAll { owner, operator } => {
                Cw1155QueryMsg::IsApprovedForAll { owner, operator }
            }
            _ => unreachable!("cannot convert {:?} to Cw1155QueryMsg", msg),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TokenInfoResponse {
    pub document_uri: String,
    pub token_uri: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct ReceiveMsg {
    /// The account that executed the send message
    pub operator: String,
    /// The account that the token transfered from
    pub from: Option<String>,
    pub token_id: TokenId,
    pub amount: Uint128,
    pub msg: Binary,
}

impl ReceiveMsg {
    /// serializes the message
    pub fn into_binary(self) -> StdResult<Binary> {
        let msg = ReceiverExecuteMsg::Receive(self);
        to_binary(&msg)
    }

    /// creates a cosmos_msg sending this struct to the named contract
    pub fn into_cosmos_msg<T: Into<String>>(self, contract_addr: T) -> StdResult<CosmosMsg> {
        let msg = self.into_binary()?;
        let execute = WasmMsg::Execute {
            contract_addr: contract_addr.into(),
            msg,
            funds: vec![],
        };
        Ok(execute.into())
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct BatchReceiveMsg {
    pub operator: String,
    pub from: Option<String>,
    pub batch: Vec<(TokenId, Uint128)>,
    pub msg: Binary,
}

impl BatchReceiveMsg {
    pub fn into_binary(self) -> StdResult<Binary> {
        let msg = ReceiverExecuteMsg::BatchReceive(self);
        to_binary(&msg)
    }

    pub fn into_cosmos_msg<T: Into<String>>(self, contract_addr: T) -> StdResult<CosmosMsg> {
        let msg = self.into_binary()?;
        let execute = WasmMsg::Execute {
            contract_addr: contract_addr.into(),
            msg,
            funds: vec![],
        };
        Ok(execute.into())
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
enum ReceiverExecuteMsg {
    Receive(ReceiveMsg),
    BatchReceive(BatchReceiveMsg),
}
