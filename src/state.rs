use candid::Principal;
use config::Config;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager};
use ic_stable_structures::DefaultMemoryImpl;
use std::cell::RefCell;

mod config;

pub const CONFIG_MEMORY_ID: MemoryId = MemoryId::new(0);

/// State of a minter canister.
#[derive(Default)]
pub struct State {
    /// Minter canister configuration.
    pub config: Config,
}

impl State {
    /// Clear the state and set initial data from settings.
    pub fn reset(&mut self, settings: Settings) {
        self.config.reset(settings);
    }
}

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
}

/// State settings.
#[derive(Debug, Clone, Copy)]
pub struct Settings {
    pub owner: Principal,
}

// impl Default for Settings {
//     fn default() -> Self {
//         Self {
//             owner: Principal::anonymous(),
//         }
//     }
// }
