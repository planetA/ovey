//! Demo for when the linux kernel module from
//! https://github.com/phip1611/generic-netlink-user-kernel-rust/
//! is loaded.

use rdma_ovey::genlink::{GenlinkAdapter, build_nl_attr, OveyAttribute, OveyOperation};

const FAMILY_NAME: &str = "CONTROL_EXMPL";

fn main() {
    let mut ga = GenlinkAdapter::connect(FAMILY_NAME);
    ga.send_single(
        OveyOperation::Echo,
        build_nl_attr(
            OveyAttribute::Msg,
            "Hello from Rust!"
        )
    );
    let recv = ga.recv_first_of_type_raw(OveyAttribute::Msg)
        .map(|bytes| String::from_utf8(bytes).unwrap())
        .unwrap();
    println!("Received: {}", recv);
    //println!("All: {:#?}", ga.recv_all());
}
