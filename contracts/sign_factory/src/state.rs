use cosmwasm_std::Addr;
use cw_storage_plus::Map;

// User -> Contract
pub const S1155_STORE: Map<&Addr, Addr> = Map::new("s1155_store");

// User -> Contracts
pub const S721_STORE: Map<&Addr, Vec<Addr>> = Map::new("s721_store");
