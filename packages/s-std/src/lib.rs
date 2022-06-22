pub mod error;
mod msg;
mod query;
mod route;

pub const NATIVE_DENOM: &str = "usign";
pub const MULTI_SIG: &str = "sign1nfvgxep88xrqza3534e92tlpnvvxctf4laa3kd";
pub const FACTORY: &str = "sign14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9sah5mss";

pub use msg::{create_fund_community_pool_msg, SignMsg, SignMsgWrapper};

pub type Response = cosmwasm_std::Response<SignMsgWrapper>;
pub type SubMsg = cosmwasm_std::SubMsg<SignMsgWrapper>;
pub type CosmosMsg = cosmwasm_std::CosmosMsg<SignMsgWrapper>;
pub type FactoryExecuteMsg = msg::FactoryExecuteMsg;

pub use query::SignQuery;
pub use route::SignRoute;

// This export is added to all contracts that import this package, signifying that they require
// "sign" support on the chain they run on.
#[no_mangle]
extern "C" fn requires_sign() {}
