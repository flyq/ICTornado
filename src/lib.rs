mod canister;
pub mod error;
mod state;

pub use crate::canister::TornadoCanister;

pub fn idl() -> String {
    let idl = TornadoCanister::idl();
    candid::bindings::candid::compile(&idl.env.env, &Some(idl.actor))
}
