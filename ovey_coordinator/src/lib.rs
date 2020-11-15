//! Public exports of ovey_coordinator. Needed for ovey_daemon.
//! Don't get confused. A Rust crate can be/export a library and a binary at the same time.
//! This works because lib.rs and main.rs are handled as separate targets.

pub const OVEY_COORDINATOR_PORT: usize = 13337;

pub mod rest; // Export the layout of *Input-Structs and *Dto-Structs.
pub mod urls; // export all urls.
pub mod data; // export type aliases
