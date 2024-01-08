use std::borrow::Cow;
use std::cell::RefCell;

use candid::{CandidType, Decode, Deserialize, Encode, Principal};
use ic_stable_structures::memory_manager::VirtualMemory;
use ic_stable_structures::storable::Bound;
use ic_stable_structures::{DefaultMemoryImpl, StableCell, Storable};

use super::Settings;
use crate::state::{CONFIG_MEMORY_ID, MEMORY_MANAGER};

/// Minter canister configuration.
#[derive(Default)]
pub struct Config {}

impl Config {
    /// Clear configuration and initialize it with data from `settings`.
    pub fn reset(&mut self, settings: Settings) {
        let new_data = ConfigData {
            owner: settings.owner,
        };
        CONFIG_CELL.with(|cell| {
            cell.borrow_mut()
                .set(new_data)
                .expect("failed to update config stable memory data")
        });
    }

    /// Returns principal of canister owner.
    pub fn get_owner(&self) -> Principal {
        CONFIG_CELL.with(|cell| cell.borrow().get().owner)
    }

    /// Sets a new principal for canister owner.
    pub fn set_owner(&mut self, owner: Principal) {
        CONFIG_CELL
            .with(|cell| cell.borrow_mut().set(ConfigData { owner }))
            .expect("failed to update config stable memory data");
    }
}

#[derive(Debug, Clone, Copy, Deserialize, CandidType, PartialEq, Eq)]
pub struct ConfigData {
    pub owner: Principal,
}

impl Default for ConfigData {
    fn default() -> Self {
        Self {
            owner: Principal::anonymous(),
        }
    }
}

impl Storable for ConfigData {
    fn to_bytes(&self) -> Cow<[u8]> {
        encode(&self).into()
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        decode(bytes.as_ref())
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 29,
        is_fixed_size: true,
    };
}

// If the struct's memory already contains a value, initializes the struct with the decoded value.
// Otherwise, sets the struct to parameter and writes it to the memory.
thread_local! {
    static CONFIG_CELL: RefCell<StableCell<ConfigData, VirtualMemory<DefaultMemoryImpl>>> = {
        RefCell::new(StableCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(CONFIG_MEMORY_ID)), ConfigData::default())
            .expect("stable memory config initialization failed"))
    };
}

pub fn encode(item: &impl CandidType) -> Vec<u8> {
    Encode!(item).expect("failed to encode item to candid")
}

pub fn decode<'a, T: CandidType + Deserialize<'a>>(bytes: &'a [u8]) -> T {
    Decode!(bytes, T).expect("failed to decode item from candid")
}
