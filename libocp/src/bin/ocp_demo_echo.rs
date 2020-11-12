//! Demo for ECHO command.

use libocp::ocp_core::Ocp;
use libocp::ocp_properties::{FAMILY_NAME, OveyAttribute, OveyOperation};

/// Demo for ECHO command.
fn main() {
    let mut ga = Ocp::connect(FAMILY_NAME, 1).unwrap();
    let res = ga.send_single_and_ack(
        OveyOperation::Echo,
        OveyAttribute::Msg,
        "Hello from Rust!"
    ).unwrap();

    println!("Received: {}", res);
}
