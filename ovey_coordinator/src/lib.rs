//! Public exports of ovey_coordinator. Needed for ovey_daemon.

pub const OVEY_COORDINATOR_PORT: usize = 13337;

pub mod rest; // Export the layout of *Input-Structs and *Dto-Structs.
pub mod urls; // export all urls.
mod data;     // without it the others don't build