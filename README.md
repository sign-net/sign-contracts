# Sign CosmWasm Contracts

Sign smart contracts are written in [CosmWasm](https://cosmwasm.com), a multi-chain smart contracting platform in Rust.

Contracts run in a WASM VM on the [Sign Layer 1 blockchain](https://github.com/sign-net/core).

## Factory

Sign's factory contract to track and manage s721 and s1155 contracts

## S-721

Sign's NFT contract s721 is a set of optional extensions on top of [cw721-base](https://github.com/CosmWasm/cw-nfts/tree/main/contracts/cw721-base), and conforms to the [cw721 specification](https://github.com/CosmWasm/cw-nfts/tree/main/packages/cw721).

## S-1155

Sign's NFT contract s1155 is a set of optional extensions on top of [cw1155-base](https://github.com/CosmWasm/cw-plus/tree/main/contracts/cw1155-base), and conforms to the [cw1155 specification](https://github.com/CosmWasm/cw-plus/tree/main/packages/cw1155).

## WasmSwap

This contract is an automatic market maker (AMM) heavily inspired by Uniswap v1 for the cosmwasm smart contract engine.

This project is fork from [wasmswap-contracts](https://github.com/Wasmswap/wasmswap-contracts).

## Build

```
make optimize
```

# DISCLAIMER

SIGN CONTRACTS IS PROVIDED “AS IS”, AT YOUR OWN RISK, AND WITHOUT WARRANTIES OF ANY KIND. No developer or entity involved in creating or instantiating Sign smart contracts will be liable for any claims or damages whatsoever associated with your use, inability to use, or your interaction with other users of Sign, including any direct, indirect, incidental, special, exemplary, punitive or consequential damages, or loss of profits, cryptocurrencies, tokens, or anything else of value. Although Sign Pte. Ltd. and it's affilliates developed the initial code for Sign, it does not own or control the Sign network, which is run by a decentralized validator set.
