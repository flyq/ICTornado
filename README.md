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