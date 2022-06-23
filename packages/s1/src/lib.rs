use cosmwasm_std::{coins, Addr, BankMsg, Decimal, MessageInfo, Uint128};
use cw_utils::must_pay;
use s_std::{create_fund_community_pool_msg, error::FeeError, SubMsg, NATIVE_DENOM};

// governance parameters
pub const OWNER_PERCENT: u64 = 95;
pub const ROYALTY_FEE: u128 = 1000; // 0.001SIGN

/// Royalty payment and distribute fees, return an error if the fee is not enough
pub fn check_royalty_payment(
    info: &MessageInfo,
    fee: u128,
    owner: Addr,
) -> Result<Vec<SubMsg>, FeeError> {
    let payment = must_pay(info, NATIVE_DENOM)?;
    if payment.u128() < fee {
        return Err(FeeError::InsufficientFee(fee, payment.u128()));
    };

    // fee will be paid only, extra payment will not be spend
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

    use crate::{check_royalty_payment, royalty_payment, FeeError, SubMsg, ROYALTY_FEE};

    #[test]
    fn test_check_royalty_payment() {
        let owner = Addr::unchecked("owner");

        // valid royalty payment
        let info = MessageInfo {
            sender: owner.clone(),
            funds: coins(ROYALTY_FEE, NATIVE_DENOM),
        };
        let result = check_royalty_payment(&info, ROYALTY_FEE, owner.clone()).unwrap();
        let bank_msg = SubMsg::new(BankMsg::Send {
            to_address: "owner".to_string(),
            amount: coins(950, NATIVE_DENOM.to_string()),
        });
        let community_msg = SubMsg::new(create_fund_community_pool_msg(coins(50, NATIVE_DENOM)));
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], bank_msg);
        assert_eq!(result[1], community_msg);

        // valid royalty payments above min fee
        let info = MessageInfo {
            sender: owner.clone(),
            funds: coins(1200, NATIVE_DENOM), // extra 200usign, but only 1000usign is spent
        };
        let result = check_royalty_payment(&info, ROYALTY_FEE, owner.clone()).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], bank_msg);
        assert_eq!(result[1], community_msg);

        // invalid payments
        let info = MessageInfo {
            sender: owner.clone(),
            funds: coins(500, NATIVE_DENOM),
        };

        // Insufficient fee
        let result = check_royalty_payment(&info, ROYALTY_FEE, owner);
        assert_eq!(result, Err(FeeError::InsufficientFee(ROYALTY_FEE, 500)));
    }

    #[test]
    fn test_royalty_payment() {
        let res = royalty_payment(ROYALTY_FEE, Addr::unchecked("owner"));
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
