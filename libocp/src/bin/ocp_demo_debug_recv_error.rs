//! Demo for DebugRespondError command.

use libocp::ocp_core::{Ocp, OcpError};

/// Demo for debug_recv_error command.
fn main() {
    let ocp = Ocp::connect().unwrap();

    for _ in 0..1 {
        let res = ocp.ocp_debug_respond_error();

        if let Err(e) = res {
            match e {
                OcpError::Invalid(_e) => {
                    println!("Successfully received error reply as expected :)");
                }
                OcpError::LowLevelError(nlerr) => {
                    panic!("Returned unexpected NlError! {}", nlerr);
                }
                _ => {
                    panic!("Here be dragons. Should not happen.");
                }
            }

        } else {
            panic!("Should result in error! Bug in Kernel or Userland?!");
        }
    }
}
