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
use crate::state::{decode, encode, StorablePrincipal, MEMORY_MANAGER, SIGNERS_MEMORY_ID};

pub mod eth;

#[derive(Deserialize, CandidType)]
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
    fn to_key_id(&self) -> EcdsaKeyId {
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

#[derive(Deserialize, CandidType)]
pub struct Signer {
    key_id: EcdsaKeyIds,
    path: Vec<u8>,
    public_key: Option<Vec<u8>>,
    chain_code: Option<Vec<u8>>,
}

impl Signer {
    pub fn new(key_id: EcdsaKeyIds, path: Vec<u8>) -> Self {
        Self {
            key_id,
            path,
            public_key: None,
            chain_code: None,
        }
    }

    pub async fn public_key(&mut self) -> Result<Vec<u8>> {
        if let Some(public_key) = &self.public_key {
            return Ok(public_key.clone());
        }

        let arg = EcdsaPublicKeyArgument {
            canister_id: None,
            derivation_path: vec![self.path.clone()],
            key_id: self.key_id.to_key_id(),
        };
        let (res,) = ecdsa_public_key(arg).await?;
        self.public_key = Some(res.public_key.clone());
        self.chain_code = Some(res.chain_code);
        Ok(res.public_key)
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
        max_size: 100,
        is_fixed_size: false,
    };
}

/// Ethereum Mainnet, Sepolia, BSC, Sol,XRP,ADA, AVAX, DOGE,
/// DOT, Polygon,Ton, ICP, SHIB, LTC, BCH, ATOM, OPtimism
pub enum CoinType {
    Evm(u64),
    Btc,
}

#[derive(Default)]
pub struct Signers {}

impl Signers {
    pub fn reset(&mut self) {
        SIGNERS.with(|signers| signers.borrow_mut().clear());
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

thread_local! {
    static SIGNERS: RefCell<StableBTreeMap<StorablePrincipal, Signer, VirtualMemory<DefaultMemoryImpl>>> = RefCell::new(StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.borrow().get(SIGNERS_MEMORY_ID))));
}
