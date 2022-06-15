echo "\n\nUploading cw20_base..."
signd tx wasm store cw20_base.wasm --gas=auto --gas-adjustment=1.15 --from validator -y
sleep 2

echo "\n\nUploading wasmswap..."
signd tx wasm store wasmswap.wasm --gas=auto --gas-adjustment=1.15 --from validator -y
sleep 2

# Init MOON token - 10k each users
export INIT=$(jq -n --arg VALIDATOR $VALIDATOR --arg USER1 $USER1 --arg USER2 $USER2 --arg USER3 $USER3 --arg USER4 $USER4 '{"name":"Moon", "symbol":"MOON", "decimals":6, "initial_balances":[{"address":$VALIDATOR, "amount":"100000000000"},{"address":$USER1, "amount":"10000000000"},{"address":$USER2, "amount":"10000000000"},{"address":$USER3, "amount":"10000000000"},{"address":$USER4, "amount":"10000000000"}], "mint":{"minter":$VALIDATOR} }')
echo "\n\nInstantiating MOON token..."
signd tx wasm instantiate 1 "$INIT" --label "Moon" --admin $VALIDATOR --gas=auto --gas-adjustment=1.15 --from validator -y
sleep 2
MOON=$(signd query wasm list-contract-by-code 1 --output json | jq -r '.contracts[-1]')

# Init swap contract
export INIT=$(jq -n --arg USIGN $DENOM --arg MOON $MOON '{"token1_denom": {"native": $USIGN }, "token2_denom": {"cw20":$MOON}, "lp_token_code_id": 1 }')
signd tx wasm instantiate 2 "$INIT" --label "USIGN MOON SWAP" --admin $VALIDATOR  --gas=auto --gas-adjustment=1.15 --from validator -y
sleep 2
USIGN_MOON_SWAP=$(signd query wasm list-contract-by-code 2 --output json | jq -r '.contracts[-1]')

# Add MOON allowance for USIGN_MOON_SWAP contract
ALLOWANCE=$(jq -n --arg USIGN_MOON_SWAP $USIGN_MOON_SWAP '{"increase_allowance":{"spender":$USIGN_MOON_SWAP, amount: "5000000000"}}')
echo "\n\nIncrease allowance for USIGN_MOON_SWAP contract..."
signd tx wasm execute $MOON $ALLOWANCE --gas=auto --gas-adjustment=1.15 --from user1 -y
sleep 2

# ADD USIGN_MOON LP 2500usign - 5000moon
ADD_LP=$(jq -n '{"add_liquidity":{"token1_amount":"2500000000","max_token2":"5000000000","min_liquidity":"1"}}')
echo "\n\nAdding USIGN_MOON liquidity..."
signd tx wasm execute $USIGN_MOON_SWAP $ADD_LP --amount 2500000000$DENOM --gas=auto --gas-adjustment=1.15 --from user1 -y
sleep 2

# BALANCE=$(jq -n --arg USER1 $VALIDATOR '{"balance":{"address":$USER1}}')
# signd query wasm contract-state smart $MOON $BALANCE

# ALL_ACC=$(jq -n '{"all_accounts":{}}')
# signd query wasm contract-state smart $MOON $ALL_ACC

# MINT=$(jq -n --arg VALIDATOR $VALIDATOR '{"mint":{"recipient":$VALIDATOR, amount: "10000000000"}}')
# signd tx wasm execute $MOON $MINT --gas=auto --gas-adjustment=1.15 --from validator -y
