use candid::{CandidType, Deserialize};
use ic_canister::{generate_idl, init, query, update, Canister, Idl, PreUpdate};
use ic_exports::candid::Principal;
use ic_exports::ic_kit::ic;

use crate::error::{Error, Result};
use crate::state::ecdsa::eth::EthWallet;
use crate::state::ecdsa::{CoinType, EcdsaKeyIds, Signer};
use crate::state::{Settings, State};

/// A canister to transfer funds between IC token canisters and EVM canister contracts.
#[derive(Canister)]
pub struct TornadoCanister {
    #[id]
    id: Principal,

    state: State,
}

impl PreUpdate for TornadoCanister {}

impl TornadoCanister {
    /// Initialize the canister with given data.
    #[init]
    pub fn init(&mut self, init_data: InitData) {
        let settings = Settings {
            owner: init_data.owner,
            ecdsa_env: init_data.ecdsa_env,
        };

        self.state.reset(settings);
    }

    /// Returns principal of canister owner.
    #[query]
    pub fn get_owner(&self) -> Principal {
        self.state.config.get_owner()
    }

    /// Sets a new principal for canister owner.
    ///
    /// This method should be called only by current owner,
    /// else `Error::NotAuthorised` will be returned.
    #[update]
    pub fn set_owner(&mut self, owner: Principal) -> Result<()> {
        self.check_owner(ic::caller())?;
        self.state.config.set_owner(owner)?;
        Ok(())
    }

    #[update]
    pub async fn init_user(&mut self) -> Result<String> {
        let caller = ic::caller();
        let signer = match self.state.signers.get(ic::caller()) {
            Some(s) => s,
            None => {
                let ecdsa_env = self.state.config.get_ecdsa_env();
                let s = Signer::new(ecdsa_env, caller.as_slice().to_vec()).await?;
                self.state.signers.set(caller, s.clone());
                s
            }
        };
        Ok(hex::encode(signer.public_key()))
    }

    #[query]
    pub fn get_address(&self, coin_type: CoinType) -> Result<String> {
        let signer = self
            .state
            .signers
            .get(ic::caller())
            .ok_or(Error::UserNotInitialized)?;

        match coin_type {
            CoinType::Evm(chain_id) => {
                let wallet = EthWallet::new(signer, chain_id)?;
                Ok(format!("{:?}", wallet.address()))
            }
            CoinType::Btc => Err(Error::Internal("not suppported".to_string())),
        }
    }

    fn check_owner(&self, principal: Principal) -> Result<()> {
        let owner = self.state.config.get_owner();
        if owner == principal || owner == Principal::anonymous() {
            return Ok(());
        }
        Err(Error::NotAuthorized)
    }

    /// Returns candid IDL.
    /// This should be the last fn to see previous endpoints in macro.
    pub fn idl() -> Idl {
        generate_idl!()
    }
}

/// Minter canister initialization data.
#[derive(Deserialize, CandidType)]
pub struct InitData {
    /// Principal of canister's owner.
    pub owner: Principal,
    pub ecdsa_env: EcdsaKeyIds,
}
