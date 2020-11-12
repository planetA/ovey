//! Demo for DebugRespondError command.

use libocp::ocp_core::Ocp;
use libocp::ocp_properties::{FAMILY_NAME, OveyOperation};

/// Demo for ECHO command.
fn main() {
    let mut ga = Ocp::connect(FAMILY_NAME, 1).unwrap();
    let res = ga.send_and_ack(
        OveyOperation::DebugRespondError,
        vec![]
    );

    if let Err(e) = res {
        println!("Successfully received error reply as expected :) - {}", e);
    } else {
        panic!("Should result in error! Bug in Kernel or Userland?!");
    }
}
