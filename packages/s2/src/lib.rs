use cosmwasm_std::{coins, Addr, BankMsg, MessageInfo};
use cw_utils::must_pay;
use s_std::{error::FeeError, SubMsg, NATIVE_DENOM};

// governance parameters
pub const MIN_FEE: u128 = 25_000_000; // 25SIGN

/// Mint payment, return an error if the fee is not enough
pub fn check_payment(info: &MessageInfo, fee: u128, multisig: Addr) -> Result<SubMsg, FeeError> {
    let payment = must_pay(info, NATIVE_DENOM)?;
    if payment.u128() < fee {
        return Err(FeeError::InsufficientFee(fee, payment.u128()));
    };

    // Payment will be spend in full
    Ok(_payment(payment.u128(), multisig))
}

/// Mint payment, assuming the right fee is passed in
fn _payment(fee: u128, multisig: Addr) -> SubMsg {
    SubMsg::new(BankMsg::Send {
        to_address: multisig.to_string(),
        amount: coins(fee, NATIVE_DENOM),
    })
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{coins, Addr, BankMsg, MessageInfo};
    use s_std::NATIVE_DENOM;

    use crate::{check_payment, FeeError, SubMsg, _payment, MIN_FEE};

    #[test]
    fn test_check_mint_payment() {
        let owner = Addr::unchecked("multisig");

        // valid min mint payment
        let info = MessageInfo {
            sender: owner.clone(),
            funds: coins(MIN_FEE, NATIVE_DENOM),
        };
        let result = check_payment(&info, MIN_FEE, owner.clone()).unwrap();
        let bank_msg = SubMsg::new(BankMsg::Send {
            to_address: owner.to_string(),
            amount: coins(MIN_FEE, NATIVE_DENOM.to_string()),
        });
        assert_eq!(result, bank_msg);

        // valid mint payment above min mint fee
        let info = MessageInfo {
            sender: owner.clone(),
            funds: coins(35_000_000, NATIVE_DENOM),
        };
        let result = check_payment(&info, MIN_FEE, owner.clone()).unwrap();
        let bank_msg = SubMsg::new(BankMsg::Send {
            to_address: owner.to_string(),
            amount: coins(35_000_000, NATIVE_DENOM.to_string()), // 35sign will be spend
        });
        assert_eq!(result, bank_msg);

        // invalid payments
        let info = MessageInfo {
            sender: owner.clone(),
            funds: coins(15_000_000, NATIVE_DENOM),
        };

        // Insufficient fee
        let result = check_payment(&info, MIN_FEE, owner);
        assert_eq!(result, Err(FeeError::InsufficientFee(MIN_FEE, 15_000_000)));
    }

    #[test]
    fn test_mint_payment() {
        let result = _payment(MIN_FEE, Addr::unchecked("multisig"));
        let bank_msg = SubMsg::new(BankMsg::Send {
            to_address: "multisig".to_string(),
            amount: coins(MIN_FEE, NATIVE_DENOM.to_string()),
        });
        assert_eq!(result, bank_msg);
    }
}
