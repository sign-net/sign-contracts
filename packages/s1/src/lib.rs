use cosmwasm_std::{coins, Addr, BankMsg, Decimal, MessageInfo, Uint128};
use cw_utils::must_pay;
use s_std::{create_fund_community_pool_msg, error::FeeError, SubMsg, NATIVE_DENOM};

// governance parameters
const OWNER_PERCENT: u64 = 95;
pub const MIN_ROYALTY_FEE: u128 = 1000; // 0.001SIGN

/// Royalty payment and distribute fees, return an error if the fee is not enough
pub fn check_royalty_payment(
    info: &MessageInfo,
    fee: u128,
    owner: Addr,
) -> Result<Vec<SubMsg>, FeeError> {
    if fee < MIN_ROYALTY_FEE {
        return Err(FeeError::BelowMinFee(
            MIN_ROYALTY_FEE,
            NATIVE_DENOM.to_string(),
        ));
    }

    let payment = must_pay(info, NATIVE_DENOM)?;
    if payment.u128() < fee {
        return Err(FeeError::InsufficientFee(fee, payment.u128()));
    };

    Ok(royalty_payment(fee, owner))
}

/// Royalty payment and distribute fees, assuming the right fee is passed in
pub fn royalty_payment(fee: u128, owner: Addr) -> Vec<SubMsg> {
    let mut msgs: Vec<SubMsg> = vec![];
    let owner_fee = (Uint128::from(fee) * Decimal::percent(OWNER_PERCENT)).u128();

    msgs.push(SubMsg::new(BankMsg::Send {
        to_address: owner.to_string(),
        amount: coins(owner_fee, NATIVE_DENOM),
    }));

    let dist_amount = fee - owner_fee;
    msgs.push(SubMsg::new(create_fund_community_pool_msg(coins(
        dist_amount,
        NATIVE_DENOM,
    ))));

    msgs
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{coins, Addr, BankMsg, MessageInfo};
    use s_std::{create_fund_community_pool_msg, NATIVE_DENOM};

    use crate::{check_royalty_payment, royalty_payment, FeeError, SubMsg, MIN_ROYALTY_FEE};

    #[test]
    fn test_check_royalty_payment() {
        let owner = Addr::unchecked("owner");

        // valid single royalty payment
        let info = MessageInfo {
            sender: owner.clone(),
            funds: coins(MIN_ROYALTY_FEE, NATIVE_DENOM),
        };
        let result = check_royalty_payment(&info, MIN_ROYALTY_FEE, owner.clone());
        assert!(result.is_ok());

        // valid 4 royalty payments
        let info = MessageInfo {
            sender: owner.clone(),
            funds: coins(4000, NATIVE_DENOM),
        };
        let result = check_royalty_payment(&info, 4000, owner.clone());
        assert!(result.is_ok());

        // invalid payments
        let info = MessageInfo {
            sender: owner.clone(),
            funds: coins(MIN_ROYALTY_FEE, NATIVE_DENOM),
        };
        // 0 fee
        let result = check_royalty_payment(&info, 0, owner.clone());
        assert_eq!(
            result,
            Err(FeeError::BelowMinFee(
                MIN_ROYALTY_FEE,
                NATIVE_DENOM.to_string()
            ))
        );
        // Insufficient fee
        let result = check_royalty_payment(&info, 2000, owner);
        assert_eq!(
            result,
            Err(FeeError::InsufficientFee(2000, MIN_ROYALTY_FEE))
        );
    }

    #[test]
    fn test_royalty_payment() {
        let res = royalty_payment(MIN_ROYALTY_FEE, Addr::unchecked("owner"));
        let bank_msg = SubMsg::new(BankMsg::Send {
            to_address: "owner".to_string(),
            amount: coins(950, NATIVE_DENOM.to_string()),
        });
        let community_msg = SubMsg::new(create_fund_community_pool_msg(coins(50, NATIVE_DENOM)));
        assert_eq!(res.len(), 2);
        assert_eq!(res[0], bank_msg);
        assert_eq!(res[1], community_msg)
    }
}
