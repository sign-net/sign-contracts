use cosmwasm_std::StdError;
use cw1155_base::ContractError as Cw1155ContractError;
use cw_utils::PaymentError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Expired")]
    Expired {},

    #[error("Token info not found")]
    TokenInfoNotFound {},

    #[error("Invalid Royalities")]
    InvalidRoyalities {},

    #[error("{0}")]
    Payment(#[from] PaymentError),

    #[error("{0}")]
    Fee(#[from] FeeError),
}

#[derive(Error, Debug, PartialEq)]
pub enum FeeError {
    #[error("Insufficient fee: expected {0}, got {1}")]
    InsufficientFee(u128, u128),

    #[error("{0}")]
    Payment(#[from] PaymentError),
}

impl From<ContractError> for Cw1155ContractError {
    fn from(err: ContractError) -> Cw1155ContractError {
        match err {
            ContractError::Std(from) => Cw1155ContractError::Std(from),
            ContractError::Unauthorized {} => Cw1155ContractError::Unauthorized {},
            ContractError::Expired {} => Cw1155ContractError::Expired {},
            _ => unreachable!("cannot convert {:?} to Cw1155ContractError", err),
        }
    }
}
