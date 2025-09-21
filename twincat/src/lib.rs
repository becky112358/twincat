mod beckhoff;
mod client;
pub use client::Client;
#[cfg(feature = "notifications")]
mod notifications;
#[cfg(feature = "notifications")]
pub use notifications::{AdsTransmissionMode, Time};
mod result;
mod rx;
mod state;
pub use state::State;
mod symbols_and_data_types;
mod tx;
mod variables;
pub use variables::{StartIndex, Variable};
mod verify;

pub use twincat_derive::path_verify;
