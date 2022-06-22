use cosmwasm_std::StdError;
use cw1155_base::ContractError as Cw1155ContractError;
use cw_utils::PaymentError;
use s_std::error::FeeError;
use thiserror::Error;
use url::ParseError;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Expired")]
    Expired {},

    #[error("Mismatch number of token and token info")]
    TokenInfoMismatch {},

    #[error("{0}")]
    Fee(#[from] FeeError),

    #[error("{0}")]
    Payment(#[from] PaymentError),

    #[error("{0}")]
    Parse(#[from] ParseError),
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
