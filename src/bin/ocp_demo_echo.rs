//! Demo for ECHO command.

use rdma_ovey::ocp;
use rdma_ovey::ocp::ocp_core::Ocp;
use rdma_ovey::ocp::ocp_properties::{OveyOperation, OveyAttribute};

/// Demo for ECHO command.
fn main() {
    let mut ga = Ocp::connect(ocp::ocp_properties::OveyOperation, 1).unwrap();
    let res = ga.send_single_and_ack(
        OveyOperation::Echo,
        OveyAttribute::Msg,
        "Hello from Rust!"
    ).unwrap();

    println!("Received: {}", res);
}
