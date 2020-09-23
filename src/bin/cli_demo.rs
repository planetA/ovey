//! This demo only tests the cli parsing without actual doing anything.

use rdma_ovey::cli::assert_and_get_args;

/// This demo only tests the cli parsing without actual doing anything.
fn main() {
    // if args are invalid this function will exit the program
    let matches = assert_and_get_args();
    println!("{:#?}", matches);
}
