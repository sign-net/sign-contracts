use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Only one S1155 contract per account is allowed.")]
    OneS1155 {},

    #[error("Contract {contract_addr} already exist")]
    AlreadyExist { contract_addr: String },
}
