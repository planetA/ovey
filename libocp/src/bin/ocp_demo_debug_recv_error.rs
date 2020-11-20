//! Demo for DebugRespondError command.

use libocp::ocp_core::Ocp;
use libocp::ocp_debug_respond_error;

/// Demo for debug_recv_error command.
fn main() {
    let mut ocp = Ocp::connect(1).unwrap();
    let res = ocp_debug_respond_error(&mut ocp);

    if let Err(e) = res {
        println!("Successfully received error reply as expected :) - {}", e);
    } else {
        panic!("Should result in error! Bug in Kernel or Userland?!");
    }
}
