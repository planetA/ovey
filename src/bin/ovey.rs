use rdma_ovey::genlink::{GenlinkAdapter, ControlAttr};

const FAMILY_NAME: &str = "rdma-ovey";

fn main() {
    let mut ga = GenlinkAdapter::connect(FAMILY_NAME);
    println!("Family id of {} is {}", FAMILY_NAME, ga.family_id());

}
