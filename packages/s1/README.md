# S1 Spec: Royalty Payment

 Royalty Payment is a specification for processing fees in Sign.

 With Royalty Payment, a portion of the fees are distributed to the owner, and the remaining portion is distributed to the community pool. 

## Governance Parameters

```rs
const OWNER_PERCENT: u64 = 95;      // 95%
const MIN_ROYALTY_FEE: u64 = 1000; // 0.001SIGN
```

## API

Contracts can use Royalty Payment via one of the following functions.

```rs
/// Royalty payment and distribute fees, return an error if the fee is not enough
pub fn check_royalty_payment(
    info: &MessageInfo,
    fee: u128,
    owner: Addr,
) -> Result<Vec<SubMsg>, FeeError>

/// Royalty payment and distribute fees, assuming the right fee is passed in
pub fn royalty_payment(fee: u128, owner: Addr) -> Vec<SubMsg>
```
