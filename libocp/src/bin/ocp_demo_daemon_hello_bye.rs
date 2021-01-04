//! Demo for ECHO command.

use libocp::ocp_core::Ocp;
use std::thread::sleep;
use std::time::Duration;
use std::process;

/// Demo for daemon hello + daemon bye command.
fn main() {
    let ocp = Ocp::connect().unwrap();

    // we also do a echo because I need to check if both sockets
    // are properly recognized at kernel
    let res_echo = ocp.ocp_echo("hello world #1");
    println!("Echo test #1 received? {}", res_echo.unwrap().msg().is_some());

    let res = ocp.ocp_daemon_hello();
    println!("my process id is: {} - check it against kernel log!", process::id());

    // we also do a echo because I need to check if both sockets
    // are properly recognized at kernel
    let res_echo = ocp.ocp_echo("hello world #2");
    println!("Echo test #2 received? {}", res_echo.unwrap().msg().is_some());

    println!("ocp daemon hello: {}", res.is_ok());
    sleep(Duration::from_millis(500));
    let res = ocp.ocp_daemon_bye();
    println!("ocp daemon bye: {}", res.is_ok());
}
