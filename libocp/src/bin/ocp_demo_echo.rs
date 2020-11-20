//! Demo for ECHO command.

use libocp::ocp_core::Ocp;

/// Demo for ECHO command.
fn main() {
    let mut ocp = Ocp::connect(1, false).unwrap();
    let res = ocp.ocp_echo("HELLO FROM RUST!!");
    let res = res.unwrap();

    println!("Received: {}", res);
}
