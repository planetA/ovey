//! Demo for ECHO command.

use libocp::ocp_core::Ocp;
use libocp::ocp_echo;

/// Demo for ECHO command.
fn main() {
    let mut ocp = Ocp::connect(1).unwrap();
    let res = ocp_echo(&mut ocp, "HELLO FROM RUST!!");
    let res = res.unwrap();

    println!("Received: {}", res);
}
