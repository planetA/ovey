//! Demo for when the linux kernel module from
//! https://github.com/phip1611/generic-netlink-user-kernel-rust/
//! is loaded.

use rdma_ovey::genlink::{GenlinkAdapter, ControlAttr};

const FAMILY_NAME: &str = "CONTROL_EXMPL";

fn main() {
    let mut ga = GenlinkAdapter::connect(FAMILY_NAME);
    ga.send(ControlAttr::Msg, "Hello World from Rust!");
    let recv = ga.recv_first_msg();
    println!("Received: {}", recv.unwrap());
    //println!("All: {:#?}", ga.recv_all());
}
