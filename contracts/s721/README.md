# s721

## Commands

Please ensure that you are running a sign chain docker node before executing the commands below. The message format can be found in the `schema` folder.

### Upload

```bash
$(echo $BINARY) tx wasm store s721.wasm --gas=auto --gas-adjustment=1.15 --from validator -y
```

### Instatiate

The contract code may not be `1` for you depending on the number of contracts you have uploaded before this.

```bash
$(echo $BINARY) tx wasm instantiate 1 '{"collection_info":{"creator":"john","description":"s721","image":"image.png","royalty_address","sign1xxxx"},"minter":"sign1xxx","name":"collection","symbol":"ABC"}' --label "s721-$USER1" --admin $USER1 --gas=auto --gas-adjustment=1.15 --from user1 -y

# Get contract address
$(echo $BINARY) query wasm list-contract-by-code 1 --output json | jq -r '.contracts[-1]'
```
