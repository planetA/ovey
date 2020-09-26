//! Demo for DebugRespondError command.

use rdma_ovey::ocp::ocp_core::Ocp;
use rdma_ovey::ocp::ocp_properties::{OveyOperation, FAMILY_NAME};

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
