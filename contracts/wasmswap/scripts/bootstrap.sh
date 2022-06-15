DENOM=usign
CHAINID=local-sign

echo "Killing existing session..."
kill -9 $(pgrep signd)
screen -S node -X quit

rm -rf ~/.sign
signd init node --chain-id $CHAINID

sed -i "s/\"os\"/\"test\"/g"  ~/.sign/config/client.toml
sed -i "s/\"\"/\"$CHAINID\"/g"  ~/.sign/config/client.toml

cp app.toml ~/.sign/config/app.toml
cp config.toml ~/.sign/config/config.toml

sed -i "s/\"stake\"/\"$DENOM\"/g" ~/.sign/config/genesis.json # set demon
sed -i "s/\"1228800\"/\"2048000\"/g" ~/.sign/config/genesis.json # max wasm code size
sed -i "s/\"-1\"/\"75000000\"/g" ~/.sign/config/genesis.json # max gas

signd keys add validator --output json > validator.json
export VALIDATOR=$(signd keys show validator -a)

signd keys add user1 --output json > user1.json
export USER1=$(signd keys show user1 -a)

signd keys add user2 --output json > user2.json
export USER2=$(signd keys show user2 -a)

signd keys add user3 --output json > user3.json
export USER3=$(signd keys show user3 -a)

signd keys add user4 --output json > user4.json
export USER4=$(signd keys show user4 -a)

signd add-genesis-account $VALIDATOR 100000000000$DENOM # 100k SIGN
signd add-genesis-account $USER1 100000000000$DENOM # 100K SIGN
signd add-genesis-account $USER2 100000000000$DENOM # 100K SIGN
signd add-genesis-account $USER3 100000000000$DENOM # 100K SIGN
signd add-genesis-account $USER4 100000000000$DENOM # 100K SIGN
signd gentx validator 10000000000$DENOM --chain-id $CHAINID # 10k SIGN  
signd collect-gentxs

screen -S node -dm signd start
echo "\nStarting node..."
sleep 8

source ./contracts.sh
source ./echo.sh
