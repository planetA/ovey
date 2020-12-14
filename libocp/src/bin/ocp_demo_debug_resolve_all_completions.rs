//! Demo bin to resolve all completions inside the kernel.
//! Useful during development if some process got stuck.

use libocp::ocp_core::Ocp;

/// Demo bin to resolve all completions inside the kernel.
/// Useful during development if some process got stuck.
fn main() {
    let mut ocp = Ocp::connect().unwrap();
    let res = ocp.ocp_debug_resolve_all_completions();
    res.unwrap();
    println!("Successfully resolved all completions in the kernel.");
}
