use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CollectionInfo {
    pub creator: String,
    pub description: String,
    pub image: String,
    pub external_link: Option<String>,
    pub royalty_address: String,
}

pub const COLLECTION_INFO: Item<CollectionInfo> = Item::new("collection_info");
