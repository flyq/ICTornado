use std::borrow::Cow;
use std::cell::RefCell;

use candid::Principal;
use candid::{CandidType, Decode, Deserialize, Encode};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager};
use ic_stable_structures::storable::Bound;
use ic_stable_structures::{DefaultMemoryImpl, Storable};

use crate::state::config::Config;
use crate::state::ecdsa::{EcdsaKeyIds, Nonces, Signers};

mod config;
pub mod ecdsa;

const CONFIG_MEMORY_ID: MemoryId = MemoryId::new(1);
const SIGNERS_MEMORY_ID: MemoryId = MemoryId::new(2);
const NONCES_MEMORY_ID: MemoryId = MemoryId::new(3);

/// State of a minter canister.
#[derive(Default)]
pub struct State {
    /// Minter canister configuration.
    pub config: Config,
    pub signers: Signers,
    pub nonces: Nonces,
}

impl State {
    /// Clear the state and set initial data from settings.
    pub fn reset(&mut self, settings: Settings) {
        self.config.reset(settings);
        self.signers.reset();
        self.nonces.reset();
    }
}

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
}

/// State settings.
#[derive(Clone, Copy)]
pub struct Settings {
    pub owner: Principal,
    pub ecdsa_env: EcdsaKeyIds,
}

#[derive(Clone, CandidType, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct StorablePrincipal(pub Principal);

impl Default for StorablePrincipal {
    fn default() -> Self {
        Self(Principal::anonymous())
    }
}

impl Storable for StorablePrincipal {
    fn to_bytes(&self) -> Cow<[u8]> {
        self.0.as_slice().into()
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Self(Principal::from_slice(&bytes))
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 29,
        is_fixed_size: false,
    };
}

pub fn encode(item: &impl CandidType) -> Vec<u8> {
    Encode!(item).expect("failed to encode item to candid")
}

pub fn decode<'a, T: CandidType + Deserialize<'a>>(bytes: &'a [u8]) -> T {
    Decode!(bytes, T).expect("failed to decode item from candid")
}
