use cosmwasm_schema::{export_schema, remove_schemas, schema_for};
use cw1155::{
    ApprovedForAllResponse, BalanceResponse, BatchBalanceResponse, TokenInfoResponse,
    TokensResponse,
};
use cw1155_base::msg::InstantiateMsg;
use s1155::msg::{ExecuteMsg, QueryMsg};
use std::env::current_dir;
use std::fs::create_dir_all;

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(TokenInfoResponse), &out_dir);

    export_schema(&schema_for!(ApprovedForAllResponse), &out_dir);
    export_schema(&schema_for!(BalanceResponse), &out_dir);
    export_schema(&schema_for!(BatchBalanceResponse), &out_dir);
    export_schema(&schema_for!(TokensResponse), &out_dir);
}
