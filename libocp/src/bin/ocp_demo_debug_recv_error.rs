//! Demo for DebugRespondError command.

use libocp::ocp_core::Ocp;

/// Demo for debug_recv_error command.
fn main() {
    let mut ocp = Ocp::connect().unwrap();
    let res = ocp.ocp_debug_respond_error();

    if let Err(e) = res {
        println!("Successfully received error reply as expected :) - {}", e);
    } else {
        panic!("Should result in error! Bug in Kernel or Userland?!");
    }
}
