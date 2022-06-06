use schemars::JsonSchema;

use serde::{Deserialize, Serialize};

/// SignRoute is enum type to represent sign query route path
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SignRoute {
    Alloc,
    Distribution,
}
