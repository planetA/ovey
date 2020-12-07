//! Demo for ECHO command.

use libocp::ocp_core::Ocp;
use std::thread::sleep;
use std::time::Duration;
use std::process;

/// Demo for daemon hello + daemon bye command.
fn main() {
    let mut ocp = Ocp::connect().unwrap();

    ocp.ocp_daemon_hello().unwrap();

    // we also do a echo because I need to check if both sockets
    // are properly recognized at kernel
    let (d_to_k_sock_reply, k_to_d_sock_reply) = ocp.ocp_debug_initiate_request();

    println!("Received reply on daemon to kernel socket for daemon initiated request? {}", d_to_k_sock_reply.is_ok());
    println!("Received request on kernel to daemon socket? {}", k_to_d_sock_reply.is_ok());




    ocp.ocp_daemon_bye().unwrap();

}
