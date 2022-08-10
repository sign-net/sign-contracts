# Sign CosmWasm Contracts

## About The Project

Sign smart contracts are written in [CosmWasm](https://cosmwasm.com), a multi-chain smart contracting platform in Rust.

Contracts run in a WASM VM on the [Sign Layer 1 blockchain](https://github.com/sign-net/core).

### Factory

Sign's factory contract to track and manage s721 and s1155 contracts

### S-721

Sign's NFT contract s721 is a set of optional extensions on top of [cw721-base](https://github.com/CosmWasm/cw-nfts/tree/main/contracts/cw721-base), and conforms to the [cw721 specification](https://github.com/CosmWasm/cw-nfts/tree/main/packages/cw721).

### S-1155

Sign's NFT contract s1155 is a set of optional extensions on top of [cw1155-base](https://github.com/CosmWasm/cw-plus/tree/main/contracts/cw1155-base), and conforms to the [cw1155 specification](https://github.com/CosmWasm/cw-plus/tree/main/packages/cw1155).

### WasmSwap

This contract is an automatic market maker (AMM) heavily inspired by Uniswap v1 for the cosmwasm smart contract engine.

This project is fork from [wasmswap-contracts](https://github.com/Wasmswap/wasmswap-contracts).

## Built With

- Rust
- [Cosmwasm](https://cosmwasm.com/)

## Get Started

### Installation

- [Rust](https://www.rust-lang.org/tools/install)

### Setting up you IDE

Do install the following plugin if you are using VSCode

- [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
- [CodeLLDB](https://marketplace.visualstudio.com/items?itemName=vadimcn.vscode-lldb)

A more detailed guide can be found [here](https://docs.cosmwasm.com/docs/1.0/getting-started/installation/#setting-up-your-ide)

### Test

Run all contracts test cases

```bash
cargo test
```

### Build

The following command will create optimized contracts for production. Please run the command at the root directory of this project.

```
make optimize
```

### Start a chain

Pull sign chain development docker image

```bash
docker pull ghcr.io/sign-net/core:latest
```

Run a container of the image

```
docker run --detach --name node -p 1317:1317 -p 26656:26656 -p 26657:26657 ghcr.io/sign-net/core:latest
```

Create variables to store commonly used commands and values.

```bash
BINARY='docker exec -i node signd'
DENOM='usign'
VALIDATOR=$($(echo $BINARY) keys show validator -a)
USER1=$($(echo $BINARY) keys show user1 -a)
USER2=$($(echo $BINARY) keys show user2 -a)
USER3=$($(echo $BINARY) keys show user3 -a)
USER4=$($(echo $BINARY) keys show user4 -a)
```

Copy wasm binaries in `artifacts` to docker container

```bash
docker cp ./contracts/wasmswap/scripts/cw20_base.wasm node:/app/cw20_base.wasm
docker cp ./artifacts/wasmswap.wasm node:/app/wasmswap.wasm

docker cp ./artifacts/s721.wasm node:/app/s721.wasm
docker cp ./artifacts/s1155.wasm node:/app/s1155.wasm
docker cp ./artifacts/sign_factory.wasm node:/app/sign_factory.wasm
```

Commands to upload, instantiate, execute and query contract and message can be found in individual contracts folder.

# DISCLAIMER

SIGN CONTRACTS IS PROVIDED “AS IS”, AT YOUR OWN RISK, AND WITHOUT WARRANTIES OF ANY KIND. No developer or entity involved in creating or instantiating Sign smart contracts will be liable for any claims or damages whatsoever associated with your use, inability to use, or your interaction with other users of Sign, including any direct, indirect, incidental, special, exemplary, punitive or consequential damages, or loss of profits, cryptocurrencies, tokens, or anything else of value. Although Sign Pte. Ltd. and it's affilliates developed the initial code for Sign, it does not own or control the Sign network, which is run by a decentralized validator set.
