# IC Tornado

## Requirement

[ic-wasm](https://github.com/dfinity/ic-wasm)
```sh
cargo install ic-wasm -f
```

## Build

```sh
cargo run  --features "export-api" > build/tornado.did

cargo build --target wasm32-unknown-unknown --release --features "export-api"

ic-wasm target/wasm32-unknown-unknown/release/tornado.wasm -o build/tornado.wasm shrink
```

## Deploy

```sh
dfx start --clean

dfx canister create --no-wallet tornado

dfx build tornado

dfx canister install tornado --argument "record { owner=principal \"$(dfx identity get-principal)\"; ecdsa_env=variant {TestKeyLocalDevelopment}}"

dfx canister install tornado --argument "record { owner=principal \"yhy6j-huy54-mkzda-m26hc-yklb3-dzz4l-i2ykq-kr7tx-dhxyf-v2c2g-tae\"; ecdsa_env=variant {TestKeyLocalDevelopment}}" --upgrade-unchanged -m=upgrade 

dfx canister call tornado get_owner

dfx canister call tornado init_user
```

dfx canister call tornado get_address '(variant {Evm= 11155111:nat64})'

dfx canister call tornado test_transfer_eth

(
  variant {
    Ok = "Signature { r: 80491585478120184467633714181782437709695235782507881035390688577328004992413, s: 28399817744222987982238971326269747786056528675566756107890547406180673768350, v: 22310257 }"
  },
)

curl https://sepolia.infura.io/v3/5b934af435ef47e0af5d57f25bc77ad0 -X POST --data '{"jsonrpc":"2.0","method":"eth_sendRawTransaction","params":["0xf86c808504e3b2920082520894bd70d89667a3e1bd341ac235259c5f2dde8172a9843b9aca00808401546d71a0762d15e56fd96cce0798a7595b29c940da7cd89ec39ea03c564ae5499fbf7c96a048fa084b91df27f862389ac8614ce76383db3e7b3d8f4c604d244e66e374afca"],"id":1}'

