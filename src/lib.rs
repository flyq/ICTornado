mod canister;
pub mod error;
pub mod state;

pub use crate::canister::TornadoCanister;

pub fn idl() -> String {
    let idl = TornadoCanister::idl();
    candid::bindings::candid::compile(&idl.env.env, &Some(idl.actor))
}

#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod tests {
    use ethers_core::k256::ecdsa::SigningKey;
    use ethers_core::types::transaction::eip2718::TypedTransaction;
    use ethers_core::types::{Address, TransactionRequest, U64};
    use ethers_signers::{Signer, Wallet};

    #[tokio::test]
    async fn signs_tx() {
        // retrieved test vector from:
        // https://web3js.readthedocs.io/en/v1.2.0/web3-eth-accounts.html#eth-accounts-signtransaction
        let tx: TypedTransaction = TransactionRequest {
            from: None,
            to: Some(
                "F0109fC8DF283027b6285cc889F5aA624EaC1F55"
                    .parse::<Address>()
                    .unwrap()
                    .into(),
            ),
            value: Some(1_000_000_000.into()),
            gas: Some(2_000_000.into()),
            nonce: Some(0.into()),
            gas_price: Some(21_000_000_000u128.into()),
            data: None,
            chain_id: Some(U64::one()),
        }
        .into();
        let wallet: Wallet<SigningKey> =
            "4c0883a69102937d6231471b5dbb6204fe5129617082792ae468d01a3f362318"
                .parse()
                .unwrap();
        let wallet = wallet.with_chain_id(tx.chain_id().unwrap().as_u64());

        let sig = wallet.sign_transaction(&tx).await.unwrap();
        let sighash = tx.sighash();
        sig.verify(sighash, wallet.address()).unwrap();

        println!("{:?}", wallet.address());
        println!("{:?}", wallet.signer().verifying_key().to_sec1_bytes());
        println!("{:?}", sig);
    }

    #[tokio::test]
    async fn signs_tx_empty_chain_id() {
        // retrieved test vector from:
        // https://web3js.readthedocs.io/en/v1.2.0/web3-eth-accounts.html#eth-accounts-signtransaction
        let tx: TypedTransaction = TransactionRequest {
            from: None,
            to: Some(
                "F0109fC8DF283027b6285cc889F5aA624EaC1F55"
                    .parse::<Address>()
                    .unwrap()
                    .into(),
            ),
            value: Some(1_000_000_000.into()),
            gas: Some(2_000_000.into()),
            nonce: Some(0.into()),
            gas_price: Some(21_000_000_000u128.into()),
            data: None,
            chain_id: None,
        }
        .into();
        let wallet: Wallet<SigningKey> =
            "4c0883a69102937d6231471b5dbb6204fe5129617082792ae468d01a3f362318"
                .parse()
                .unwrap();
        let wallet = wallet.with_chain_id(1u64);

        // this should populate the tx chain_id as the signer's chain_id (1) before signing
        let sig = wallet.sign_transaction(&tx).await.unwrap();

        // since we initialize with None we need to re-set the chain_id for the sighash to be
        // correct
        let mut tx = tx;
        tx.set_chain_id(1);
        let sighash = tx.sighash();
        sig.verify(sighash, wallet.address()).unwrap();

        println!("{:?}", sig);
    }

    #[tokio::test]
    async fn signs_tx_empty_chain_id_sync() {
        use ethers_core::types::TransactionRequest;

        let chain_id = 1337u64;
        // retrieved test vector from:
        // https://web3js.readthedocs.io/en/v1.2.0/web3-eth-accounts.html#eth-accounts-signtransaction
        let tx: TypedTransaction = TransactionRequest {
            from: None,
            to: Some(
                "F0109fC8DF283027b6285cc889F5aA624EaC1F55"
                    .parse::<Address>()
                    .unwrap()
                    .into(),
            ),
            value: Some(1_000_000_000u64.into()),
            gas: Some(2_000_000u64.into()),
            nonce: Some(0u64.into()),
            gas_price: Some(21_000_000_000u128.into()),
            data: None,
            chain_id: None,
        }
        .into();
        let wallet: Wallet<SigningKey> =
            "4c0883a69102937d6231471b5dbb6204fe5129617082792ae468d01a3f362318"
                .parse()
                .unwrap();
        let wallet = wallet.with_chain_id(chain_id);

        // this should populate the tx chain_id as the signer's chain_id (1337) before signing and
        // normalize the v
        let sig = wallet.sign_transaction_sync(&tx).unwrap();

        // ensure correct v given the chain - first extract recid
        let recid = (sig.v - 35) % 2;
        // eip155 check
        assert_eq!(sig.v, chain_id * 2 + 35 + recid);

        // since we initialize with None we need to re-set the chain_id for the sighash to be
        // correct
        let mut tx = tx;
        tx.set_chain_id(chain_id);
        let sighash = tx.sighash();
        sig.verify(sighash, wallet.address()).unwrap();

        println!("{:?}", sig);
    }
}
