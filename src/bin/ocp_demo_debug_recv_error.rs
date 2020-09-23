//! Demo for DebugRespondError command.

use rdma_ovey::ocp;
use rdma_ovey::ocp::ocp_core::Ocp;
use rdma_ovey::ocp::ocp_properties::{OveyOperation, OveyAttribute};

/// Demo for ECHO command.
fn main() {
    let mut ga = Ocp::connect(ocp::ocp_properties::OveyOperation, 1).unwrap();
    let res = ga.send_and_ack(
        OveyOperation::DebugRespondError,
        vec![]
    ).unwrap();

    if let Err(e) = res {
        println!("Successfully received error reply as expected :) - {}", e);
    } else {
        panic!("Should result in error! Bug in Kernel or Userland?!");
    }
}
