use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::route::SignRoute;
use cosmwasm_std::{Coin, CosmosMsg, CustomMsg};
use cw721::CustomMsg as Cw721CustomMsg;
static MSG_DATA_VERSION: &str = "1.0.0";

/// SignMsg is an override of CosmosMsg::Custom to add support for Sign's custom message types
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct SignMsgWrapper {
    pub route: SignRoute,
    pub msg_data: SignMsg,
    pub version: String,
}

impl From<SignMsgWrapper> for CosmosMsg<SignMsgWrapper> {
    fn from(original: SignMsgWrapper) -> Self {
        CosmosMsg::Custom(original)
    }
}

impl CustomMsg for SignMsgWrapper {}
impl Cw721CustomMsg for SignMsgWrapper {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SignMsg {
    FundCommunityPool { amount: Vec<Coin> },
}

pub fn create_fund_community_pool_msg(amount: Vec<Coin>) -> CosmosMsg<SignMsgWrapper> {
    SignMsgWrapper {
        route: SignRoute::Distribution,
        msg_data: SignMsg::FundCommunityPool { amount },
        version: MSG_DATA_VERSION.to_owned(),
    }
    .into()
}
