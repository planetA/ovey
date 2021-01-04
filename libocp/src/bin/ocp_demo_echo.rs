//! Demo for ECHO command.

use libocp::ocp_core::Ocp;

/// Demo for ECHO command.
fn main() {
    let ocp = Ocp::connect().unwrap();
    let res = ocp.ocp_echo("HELLO FROM RUST!!");
    let res = res.unwrap();

    println!("Received: {}", res.msg().expect("Must receive echo msg"));
}
