echo "\n\nUploading cw20_base..."
signd tx wasm store cw20_base.wasm --gas=auto --gas-adjustment=1.15 --from user1 -y
sleep 2

echo "\n\nUploading wasmswap..."
signd tx wasm store wasmswap.wasm --gas=auto --gas-adjustment=1.15 --from user1 -y
sleep 2


# Init MOON token - 10k each users
export INIT=$(jq -n --arg VALIDATOR $VALIDATOR --arg USER1 $USER1 --arg USER2 $USER2 --arg USER3 $USER3 --arg USER4 $USER4 '{"name":"Moon", "symbol":"MOON", "decimals":6, "initial_balances":[{"address":$VALIDATOR, "amount":"100000000000"},{"address":$USER1, "amount":"10000000000"},{"address":$USER2, "amount":"10000000000"},{"address":$USER3, "amount":"10000000000"},{"address":$USER4, "amount":"10000000000"}], "mint":{"minter":$USER1} }')
echo "\n\nInstantiating MOON token..."
signd tx wasm instantiate 1 "$INIT" --label "Moon" --admin $USER1 --gas=auto --gas-adjustment=1.15 --from user1 -y
sleep 2
MOON=$(signd query wasm list-contract-by-code 1 --output json | jq -r '.contracts[-1]')

# Init SUN token - 10k each users
export INIT=$(jq -n --arg VALIDATOR $VALIDATOR --arg USER1 $USER1 --arg USER2 $USER2 --arg USER3 $USER3 --arg USER4 $USER4 '{"name":"Sun", "symbol":"SUN", "decimals":6, "initial_balances":[{"address":$VALIDATOR, "amount":"100000000000"},{"address":$USER1, "amount":"10000000000"},{"address":$USER2, "amount":"10000000000"},{"address":$USER3, "amount":"10000000000"},{"address":$USER4, "amount":"10000000000"}], "mint":{"minter":$USER1} }')
echo "\n\nInstantiating SUN token..."
signd tx wasm instantiate 1 "$INIT" --label "Sun" --admin $USER1 --gas=auto --gas-adjustment=1.15 --from user1 -y
sleep 2
SUN=$(signd query wasm list-contract-by-code 1 --output json | jq -r '.contracts[-1]')


# Init USIGN<->MOON contract
export INIT=$(jq -n --arg USIGN $DENOM --arg MOON $MOON '{"token1_denom": {"native": $USIGN }, "token2_denom": {"cw20":$MOON}, "lp_token_code_id": 1 }')
echo "\n\nInstantiating USIGN<->MOON swap contract..."
signd tx wasm instantiate 2 "$INIT" --label "USIGN MOON SWAP" --admin $USER1  --gas=auto --gas-adjustment=1.15 --from user1 -y
sleep 2
USIGN_MOON_SWAP=$(signd query wasm list-contract-by-code 2 --output json | jq -r '.contracts[-1]')

# Add MOON allowance for USIGN_MOON_SWAP contract
ALLOWANCE=$(jq -n --arg USIGN_MOON_SWAP $USIGN_MOON_SWAP '{"increase_allowance":{"spender":$USIGN_MOON_SWAP, amount: "5000000000"}}')
echo "\n\nIncrease allowance for USIGN_MOON_SWAP contract..."
signd tx wasm execute $MOON $ALLOWANCE --gas=auto --gas-adjustment=1.15 --from user1 -y
sleep 2

# ADD USIGN_MOON LP 2500USIGN - 5000MOON
ADD_LP=$(jq -n '{"add_liquidity":{"token1_amount":"2500000000","max_token2":"5000000000","min_liquidity":"1"}}')
echo "\n\nAdding USIGN<->MOON liquidity..."
signd tx wasm execute $USIGN_MOON_SWAP $ADD_LP --amount 2500000000$DENOM --gas=auto --gas-adjustment=1.15 --from user1 -y
sleep 2



# Init USIGN<->SUN contract
export INIT=$(jq -n --arg USIGN $DENOM --arg SUN $SUN '{"token1_denom": {"native": $USIGN }, "token2_denom": {"cw20":$SUN}, "lp_token_code_id": 1 }')
echo "\n\nInstantiating USIGN<->SUN swap contract..."
signd tx wasm instantiate 2 "$INIT" --label "USIGN SUN SWAP" --admin $USER1  --gas=auto --gas-adjustment=1.15 --from user1 -y
sleep 2
USIGN_SUN_SWAP=$(signd query wasm list-contract-by-code 2 --output json | jq -r '.contracts[-1]')

# Add SUN allowance for USIGN_SUN_SWAP contract
ALLOWANCE=$(jq -n --arg USIGN_SUN_SWAP $USIGN_SUN_SWAP '{"increase_allowance":{"spender":$USIGN_SUN_SWAP, amount: "1000000000"}}')
echo "\n\nIncrease allowance for USIGN_SUN_SWAP contract..."
signd tx wasm execute $SUN $ALLOWANCE --gas=auto --gas-adjustment=1.15 --from user1 -y
sleep 2

# ADD USIGN_SUN LP 500USIGN - 1000SUN
ADD_LP=$(jq -n '{"add_liquidity":{"token1_amount":"500000000","max_token2":"1000000000","min_liquidity":"1"}}')
echo "\n\nAdding USIGN<->SUN liquidity..."
signd tx wasm execute $USIGN_SUN_SWAP $ADD_LP --amount 500000000$DENOM --gas=auto --gas-adjustment=1.15 --from user1 -y
sleep 2



# Init MOON<->SUN contract
export INIT=$(jq -n --arg MOON $MOON --arg SUN $SUN '{"token1_denom": {"cw20": $MOON }, "token2_denom": {"cw20":$SUN}, "lp_token_code_id": 1 }')
echo "\n\nInstantiating SUN<->MOON swap contract..."
signd tx wasm instantiate 2 "$INIT" --label "MOON SUN SWAP" --admin $USER1  --gas=auto --gas-adjustment=1.15 --from user1 -y
sleep 2
MOON_SUN_SWAP=$(signd query wasm list-contract-by-code 2 --output json | jq -r '.contracts[-1]')

# Add MOON allowance for USIGN_MOON_SWAP contract
ALLOWANCE=$(jq -n --arg MOON_SUN_SWAP $MOON_SUN_SWAP '{"increase_allowance":{"spender":$MOON_SUN_SWAP, amount: "500000000"}}')
echo "\n\nIncrease MOON allowance for MOON_SUN_SWAP contract..."
signd tx wasm execute $MOON $ALLOWANCE --gas=auto --gas-adjustment=1.15 --from user1 -y
sleep 2

# Add SUN allowance for USIGN_SUN_SWAP contract
ALLOWANCE=$(jq -n --arg MOON_SUN_SWAP $MOON_SUN_SWAP '{"increase_allowance":{"spender":$MOON_SUN_SWAP, amount: "500000000"}}')
echo "\n\nIncrease SUN allowance for MOON_SUN_SWAP contract..."
signd tx wasm execute $SUN $ALLOWANCE --gas=auto --gas-adjustment=1.15 --from user1 -y
sleep 2

# ADD MOON_SUN LP 500MOON - 500SUN
ADD_LP=$(jq -n '{"add_liquidity":{"token1_amount":"500000000","max_token2":"500000000","min_liquidity":"1"}}')
echo "\n\nAdding MOON<->SUN liquidity..."
signd tx wasm execute $MOON_SUN_SWAP $ADD_LP --gas=auto --gas-adjustment=1.15 --from user1 -y
sleep 2


# BALANCE=$(jq -n --arg USER1 $VALIDATOR '{"balance":{"address":$USER1}}')
# signd query wasm contract-state smart $MOON $BALANCE

# ALL_ACC=$(jq -n '{"all_accounts":{}}')
# signd query wasm contract-state smart $MOON $ALL_ACC

# MINT=$(jq -n --arg VALIDATOR $VALIDATOR '{"mint":{"recipient":$VALIDATOR, amount: "10000000000"}}')
# signd tx wasm execute $MOON $MINT --gas=auto --gas-adjustment=1.15 --from validator -y
