mod derive;
mod rpc;

pub use derive::{derive_from_seed, validate_address, BtcSigner};
pub use rpc::BtcRpc;
