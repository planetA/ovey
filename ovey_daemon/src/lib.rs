//! API of Ovey Daemon. Exports necessary information for the REST API for Ovey CLI.
//! Ovey CLI uses the daemon to create and delete devices in the kernel.

// all public information for ovey cli (uses this as dependency/lib)
mod cli_rest_api;

pub use cli_rest_api::*;

// We re-export ovey DTOs because Ovey Daemon pass them through as they are
pub use ovey_coordinator::rest as coordinator_rest;
