# S2 Spec: Mint Payment

Mint Payment is a specification for processing fees for minting NDF in Sign.

With Mint Payment, the fees will be transferred to a designated multi-sig account.

## Governance Parameters

```rs
const MIN_MINT_FEE: u128 = 25_000_000; // 25SIGN
```

## API

Contracts can use Mint Payment via one of the following functions.

```rs
/// Mint payment, return an error if the fee is not enough
pub fn check_mint_payment(
    info: &MessageInfo,
    fee: u128,
    multisig: Addr,
) -> Result<Vec<SubMsg>, FeeError> 

/// Mint payment, assuming the right fee is passed in
pub fn mint_payment(fee: u128, multisig: Addr) -> Vec<SubMsg>
```
