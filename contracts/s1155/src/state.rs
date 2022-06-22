use cosmwasm_std::Addr;
use cw_storage_plus::Item;

/// Address to send royalty payment to.
pub const ROYALTY: Item<Addr> = Item::new("royalty");
