VALIDATOR=$($(echo $BINARY) keys show validator -a)
USER1=$($(echo $BINARY) keys show user1 -a)
USER2=$($(echo $BINARY) keys show user2 -a)
USER3=$($(echo $BINARY) keys show user3 -a)
USER4=$($(echo $BINARY) keys show user4 -a)

MOON=$($(echo $BINARY)  query wasm list-contract-by-code 1 --output json | jq -r '.contracts[-5]')
SUN=$($(echo $BINARY)  query wasm list-contract-by-code 1 --output json | jq -r '.contracts[-4]')
USIGN_MOON_TOKEN=$($(echo $BINARY)  query wasm list-contract-by-code 1 --output json | jq -r '.contracts[-3]')
USIGN_SUN_TOKEN=$($(echo $BINARY)  query wasm list-contract-by-code 1 --output json | jq -r '.contracts[-2]')
MOON_SUN_TOKEN=$($(echo $BINARY)  query wasm list-contract-by-code 1 --output json | jq -r '.contracts[-1]')

USIGN_MOON_SWAP=$($(echo $BINARY)  query wasm list-contract-by-code 2 --output json | jq -r '.contracts[-3]')
USIGN_SUN_SWAP=$($(echo $BINARY)  query wasm list-contract-by-code 2 --output json | jq -r '.contracts[-2]')
MOON_SUN_SWAP=$($(echo $BINARY)  query wasm list-contract-by-code 2 --output json | jq -r '.contracts[-1]')

echo "\nAccounts"
echo "VALIDATOR: $VALIDATOR"
echo "USER1: $USER1"
echo "USER2: $USER2"
echo "USER3: $USER3"
echo "USER4: $USER4"

echo "\nCW20 tokens"
echo "MOON: $MOON"
echo "SUN: $SUN"
echo "USIGN_MOON_TOKEN: $USIGN_MOON_TOKEN"
echo "USIGN_SUN_TOKEN: $USIGN_SUN_TOKEN"
echo "MOON_SUN_TOKEN: $MOON_SUN_TOKEN"


echo "\nPairs"
echo "USIGN_MOON_SWAP: $USIGN_MOON_SWAP"
echo "USIGN_SUN_SWAP: $USIGN_SUN_SWAP"
echo "MOON_SUN_SWAP: $MOON_SUN_SWAP"
