//! Demo for ECHO command.

use libocp::ocp_core::Ocp;
use std::thread::sleep;
use std::time::Duration;
use std::process;

/// Demo for daemon hello + daemon bye command.
fn main() {
    let ocp = Ocp::connect().unwrap();

    let res = ocp.ocp_daemon_hello();
    println!("my process id is: {} - check it against kernel log!", process::id());

    println!("ocp daemon hello: {}", res.is_ok());
    sleep(Duration::from_millis(500));
    let res = ocp.ocp_daemon_bye();
    println!("ocp daemon bye: {}", res.is_ok());
}
