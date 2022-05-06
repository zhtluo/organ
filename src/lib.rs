//! This is a prototype implementation of the OrgAn protocol proposed in the
//! paper 'OrgAn: Organizational Anonymity with Low Latency'. The protocol
//! follows a client/relay/server model, where the setup server provides secret
//! shares of a publicly known value to the clients. The clients in the
//! organisation communicate anonymously through the relay with the outside
//! world. The communication proceeds in Base and Bulk rounds.
#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

/// Crate for log functionalities.
#[macro_use]
extern crate log;

/// Handles client-side communication.
pub mod client;
/// Handles config file read/write.
pub mod config;
/// Handles elliptic curve computation.
pub mod ecc;
/// Handles flint-related native operations.
pub mod flint;
/// Handles guard node setup operation.
pub mod guard;
/// Handles message formatting.
pub mod message;
/// Handles network-related functionalities.
pub mod net;
/// Handles additional on-demand PRF computation.
pub mod prf;
/// Handles server-side communication.
pub mod server;
