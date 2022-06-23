# S2 Spec: Payment

Payment is a specification for processing fees for creating contracts or minting NDF in Sign.

With Payment, the fees will be transferred to a designated multi-sig account.

## Governance Parameters

```rs
const MIN_FEE: u128 = 25_000_000; // 25SIGN
```

## API

Contracts can use Payment via the following function.

```rs
/// Mint payment, return an error if the fee is not enough
pub fn check_payment(
    info: &MessageInfo,
    fee: u128,
    multisig: Addr,
) -> Result<SubMsg, FeeError>
```
