use cosmwasm_std::{coins, Addr, BankMsg, MessageInfo};
use cw_utils::must_pay;
use s_std::{error::FeeError, SubMsg, NATIVE_DENOM};

// governance parameters
pub const MIN_MINT_FEE: u128 = 25_000_000; // 25SIGN

/// Mint payment, return an error if the fee is not enough
pub fn check_mint_payment(
    info: &MessageInfo,
    fee: u128,
    multisig: Addr,
) -> Result<Vec<SubMsg>, FeeError> {
    let payment = must_pay(info, NATIVE_DENOM)?;
    if payment.u128() < fee {
        return Err(FeeError::InsufficientFee(fee, payment.u128()));
    };

    Ok(mint_payment(fee, multisig))
}

/// Mint payment, assuming the right fee is passed in
pub fn mint_payment(fee: u128, multisig: Addr) -> Vec<SubMsg> {
    let msgs: Vec<SubMsg> = vec![SubMsg::new(BankMsg::Send {
        to_address: multisig.to_string(),
        amount: coins(fee, NATIVE_DENOM),
    })];

    msgs
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{coins, Addr, BankMsg, MessageInfo};
    use s_std::NATIVE_DENOM;

    use crate::{check_mint_payment, mint_payment, FeeError, SubMsg, MIN_MINT_FEE};

    #[test]
    fn test_check_mint_payment() {
        let owner = Addr::unchecked("multisig");

        // valid min mint payment
        let info = MessageInfo {
            sender: owner.clone(),
            funds: coins(MIN_MINT_FEE, NATIVE_DENOM),
        };
        let result = check_mint_payment(&info, MIN_MINT_FEE, owner.clone());
        assert!(result.is_ok());

        // valid mint payment
        let info = MessageInfo {
            sender: owner.clone(),
            funds: coins(35_000_000, NATIVE_DENOM),
        };
        let result = check_mint_payment(&info, 35_000_000, owner.clone());
        assert!(result.is_ok());

        // invalid payments
        let info = MessageInfo {
            sender: owner.clone(),
            funds: coins(MIN_MINT_FEE, NATIVE_DENOM),
        };

        // Insufficient fee
        let result = check_mint_payment(&info, 30_000_000, owner);
        assert_eq!(
            result,
            Err(FeeError::InsufficientFee(30_000_000, MIN_MINT_FEE))
        );
    }

    #[test]
    fn test_mint_payment() {
        let result = mint_payment(MIN_MINT_FEE, Addr::unchecked("multisig"));
        let bank_msg = SubMsg::new(BankMsg::Send {
            to_address: "multisig".to_string(),
            amount: coins(MIN_MINT_FEE, NATIVE_DENOM.to_string()),
        });
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], bank_msg);
    }
}
