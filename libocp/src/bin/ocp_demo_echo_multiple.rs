//! Demo for multiple ECHO command.

use libocp::ocp_core::Ocp;

/// Demo for multiple ECHO command.
fn main() {
    let ocp = Ocp::connect().unwrap();

    for _ in 0..1000 {
        let res = ocp.ocp_echo("HELLO FROM RUST!!").unwrap();
        assert_eq!("HELLO FROM RUST!!", res.msg().unwrap())
    }

}
