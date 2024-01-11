use ethers_core::k256::elliptic_curve::sec1::ToEncodedPoint;
use ethers_core::k256::PublicKey;
use ethers_core::types::transaction::eip2718::TypedTransaction;
use ethers_core::types::{Address, Signature, H256, U256};
use tiny_keccak::{Hasher, Keccak};

use crate::error::{Error, Result};
use crate::state::ecdsa::Signer;

pub struct EthWallet {
    pub signer: Signer,
    pub address: Address,
    pub chain_id: u64,
}

impl EthWallet {
    pub fn new(signer: Signer, chain_id: u64) -> Result<Self> {
        let address = public_key_to_address(signer.public_key())?;
        Ok(Self {
            signer,
            address,
            chain_id,
        })
    }

    pub async fn sign_transaction(&self, tx: &TypedTransaction) -> Result<Signature> {
        let mut tx = tx.clone();
        if tx.chain_id().is_none() {
            // in the case we don't have a chain_id, let's use the signer chain id instead
            tx.set_chain_id(self.chain_id);
        }

        let sighash = tx.sighash();
        let mut sig = self.sign_hash(sighash).await?;

        // sign_hash sets `v` to recid + 27, so we need to subtract 27 before normalizing
        sig.v = to_eip155_v(sig.v as u8 - 27, self.chain_id);

        Ok(sig)
    }

    // Signs the provided hash.
    async fn sign_hash(&self, hash: H256) -> Result<Signature> {
        let sign = self.signer.sign_hash(hash.0).await?;

        let v = 27;

        let r_bytes = &sign[0..32];
        let s_bytes = &sign[32..];
        let r = U256::from_big_endian(r_bytes);
        let s = U256::from_big_endian(s_bytes);

        Ok(Signature { r, s, v })
    }

    pub fn address(&self) -> Address {
        self.address
    }
}

/// Compute the Keccak-256 hash of input bytes.
///
/// Note that strings are interpreted as UTF-8 bytes,
// TODO: Add Solidity Keccak256 packing support
pub fn keccak256<T: AsRef<[u8]>>(bytes: T) -> [u8; 32] {
    let mut output = [0u8; 32];

    let mut hasher = Keccak::v256();
    hasher.update(bytes.as_ref());
    hasher.finalize(&mut output);

    output
}

/// Applies [EIP155](https://github.com/ethereum/EIPs/blob/master/EIPS/eip-155.md)
pub fn to_eip155_v<T: Into<u8>>(recovery_id: T, chain_id: u64) -> u64 {
    (recovery_id.into() as u64) + chain_id * 2 + 35
}

/// Convert a raw, uncompressed public key to an address.
/// the public's length should be 33
pub fn public_key_to_address(pubkey: &[u8]) -> Result<Address> {
    let uncompressed_public_key = PublicKey::from_sec1_bytes(pubkey)
        .map_err(|_| Error::InvalidPublicKey(hex::encode(pubkey)))?;

    let public_key = uncompressed_public_key.to_encoded_point(false);
    let public_key = public_key.as_bytes();
    debug_assert_eq!(public_key[0], 0x04);
    let hash = keccak256(&public_key[1..]);

    let mut bytes = [0u8; 20];
    bytes.copy_from_slice(&hash[12..]);
    Ok(Address::from_slice(&bytes))
}
