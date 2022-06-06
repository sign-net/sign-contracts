use cosmwasm_std::{Addr, Decimal, Uint128};
use cw1155::Expiration;
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TokenInfo<T> {
    pub image_uri: String,
    pub document_uri: String,
    pub token_uri: String,
    pub royalty_info: Option<T>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RoyaltyInfo {
    pub payment_address: Addr,
    pub share: Decimal,
}

/// Store the minter address who have permission to mint new tokens.
pub const MINTER: Item<Addr> = Item::new("minter");
/// Store the balance map, `(owner, token_id) -> balance`
pub const BALANCES: Map<(&Addr, &str), Uint128> = Map::new("balances");
/// Store the approval status, `(owner, spender) -> expiration`
pub const APPROVES: Map<(&Addr, &Addr), Expiration> = Map::new("approves");
/// Store TokenInfo<T> data, also supports enumerating tokens,
/// An entry for token_id must exist as long as there's tokens in circulation.
pub const TOKENS: Map<&str, TokenInfo<RoyaltyInfo>> = Map::new("tokens");
