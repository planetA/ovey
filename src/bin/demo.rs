//! Demo for when the linux kernel module from
//! https://github.com/phip1611/generic-netlink-user-kernel-rust/
//! is loaded.

use rdma_ovey::ocp::{Ocp, OveyAttribute, OveyOperation};

const FAMILY_NAME: &str = "CONTROL_EXMPL";

fn main() {
    let mut ga = Ocp::connect(FAMILY_NAME, 1).unwrap();
    let res = ga.send_single_and_ack(
        OveyOperation::Echo,
        OveyAttribute::Msg,
        "Hello from Rust!"
    ).unwrap();

    println!("Received: {}", res);
    //println!("All: {:#?}", ga.recv_all());
}
