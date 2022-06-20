use cosmwasm_std::{Addr, CanonicalAddr};
use cw_storage_plus::Map;

pub const S1155_STORE: Map<&Addr, CanonicalAddr> = Map::new("s1155_store");
pub const S721_STORE: Map<&Addr, Vec<CanonicalAddr>> = Map::new("s721_store");
