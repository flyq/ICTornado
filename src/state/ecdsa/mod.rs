use std::borrow::Cow;
use std::cell::RefCell;

use candid::{CandidType, Deserialize, Principal};
use ic_exports::ic_cdk::api::management_canister::ecdsa::{
    ecdsa_public_key, sign_with_ecdsa, EcdsaCurve, EcdsaKeyId, EcdsaPublicKeyArgument,
    SignWithEcdsaArgument,
};
use ic_stable_structures::memory_manager::VirtualMemory;
use ic_stable_structures::storable::Bound;
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap, Storable};

use crate::error::Result;
use crate::state::{
    decode, encode, StorablePrincipal, MEMORY_MANAGER, NONCES_MEMORY_ID, SIGNERS_MEMORY_ID,
};

pub mod eth;

#[derive(Copy, Clone, Deserialize, CandidType)]
pub enum EcdsaKeyIds {
    TestKeyLocalDevelopment,
    TestKey1,
    ProductionKey1,
}

/// (secp256k1, test_key_1): the test key deployed on a single 13-node subnet.
///
/// (secp256k1, key_1) the production key deployed on two high-replication subnets, one activated for signing,
/// and the other one for backing up the key for better key availability.
/// https://internetcomputer.org/docs/current/developer-docs/integrations/t-ecdsa/t-ecdsa-how-it-works#ecdsa-keys
impl EcdsaKeyIds {
    fn to_key_id(self) -> EcdsaKeyId {
        EcdsaKeyId {
            curve: EcdsaCurve::Secp256k1,
            name: match self {
                Self::TestKeyLocalDevelopment => "dfx_test_key",
                Self::TestKey1 => "test_key_1",
                Self::ProductionKey1 => "key_1",
            }
            .to_string(),
        }
    }
}

// if change the struct, need to update the BOUND in Storable impl
#[derive(Clone, CandidType, Deserialize)]
pub struct Signer {
    key_id: EcdsaKeyIds,
    path: Vec<u8>,
    public_key: Vec<u8>,
    chain_code: Vec<u8>,
}

impl Signer {
    pub async fn new(key_id: EcdsaKeyIds, path: Vec<u8>) -> Result<Self> {
        let arg = EcdsaPublicKeyArgument {
            canister_id: None,
            derivation_path: vec![path.clone()],
            key_id: key_id.to_key_id(),
        };
        let (res,) = ecdsa_public_key(arg).await?;

        Ok(Self {
            key_id,
            path,
            public_key: res.public_key,
            chain_code: res.chain_code,
        })
    }

    pub fn public_key(&self) -> &[u8] {
        &self.public_key
    }

    pub fn chain_code(&self) -> &[u8] {
        &self.chain_code
    }

    pub async fn sign_hash(&self, hash: [u8; 32]) -> Result<Vec<u8>> {
        let arg = SignWithEcdsaArgument {
            message_hash: hash.to_vec(),
            derivation_path: vec![self.path.clone()],
            key_id: self.key_id.to_key_id(),
        };
        let (res,) = sign_with_ecdsa(arg).await?;
        Ok(res.signature)
    }
}

impl Storable for Signer {
    fn to_bytes(&self) -> Cow<[u8]> {
        encode(&self).into()
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        decode(bytes.as_ref())
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 153,
        is_fixed_size: false,
    };
}

/// Ethereum Mainnet, Sepolia, BSC, Sol,XRP,ADA, AVAX, DOGE,
/// DOT, Polygon,Ton, ICP, SHIB, LTC, BCH, ATOM, OPtimism
#[derive(Clone, CandidType, Deserialize)]
pub enum CoinType {
    Evm(u64),
    Btc,
}

#[derive(Default, Clone, Copy)]
pub struct Signers {}

impl Signers {
    pub fn reset(&mut self) {
        SIGNERS.with(|signers| {
            signers.replace(StableBTreeMap::new(
                MEMORY_MANAGER.with(|m| m.borrow().get(SIGNERS_MEMORY_ID)),
            ))
        });
    }

    pub fn get(&self, principal: Principal) -> Option<Signer> {
        SIGNERS.with(|signers| signers.borrow().get(&StorablePrincipal(principal)))
    }

    pub fn set(&mut self, principal: Principal, signer: Signer) {
        SIGNERS.with(|signers| {
            signers
                .borrow_mut()
                .insert(StorablePrincipal(principal), signer)
        });
    }
}

#[derive(Clone, CandidType, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct PrincipalChainIdKey(pub Principal, pub u64); // user + chain id

impl Default for PrincipalChainIdKey {
    fn default() -> Self {
        Self(Principal::anonymous(), 0)
    }
}

impl Storable for PrincipalChainIdKey {
    fn to_bytes(&self) -> Cow<[u8]> {
        encode(&self).into()
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        decode(bytes.as_ref())
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 29,
        is_fixed_size: false,
    };
}

#[derive(Default, Clone, Copy)]
pub struct Nonces {}

impl Nonces {
    pub fn reset(&mut self) {
        NONCES.with(|nonces| {
            nonces.replace(StableBTreeMap::new(
                MEMORY_MANAGER.with(|m| m.borrow().get(NONCES_MEMORY_ID)),
            ))
        });
    }

    pub fn get(&self, principal: Principal, chain_id: u64) -> Option<u64> {
        NONCES.with(|nonces| {
            nonces
                .borrow()
                .get(&PrincipalChainIdKey(principal, chain_id))
        })
    }

    pub fn set(&mut self, principal: Principal, chain_id: u64, nonce: u64) {
        NONCES.with(|nonces| {
            nonces
                .borrow_mut()
                .insert(PrincipalChainIdKey(principal, chain_id), nonce)
        });
    }
}

thread_local! {
    static SIGNERS: RefCell<StableBTreeMap<StorablePrincipal, Signer, VirtualMemory<DefaultMemoryImpl>>> = RefCell::new(StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.borrow().get(SIGNERS_MEMORY_ID))));
    static NONCES: RefCell<StableBTreeMap<PrincipalChainIdKey, u64, VirtualMemory<DefaultMemoryImpl>>> = RefCell::new(StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.borrow().get(NONCES_MEMORY_ID))))
}
