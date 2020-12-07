//! Demo that uses OCP against the Ovey Kernel module to test some functionality.
//! Make sure that this version of ocp_properties matches the one inside the kernel module!

use libocp::ocp_core::{Ocp};

/// Demo for delete command.
fn main() {
    let mut ocp = Ocp::connect().unwrap();

    let device_name = "ovey0".to_string();

    let res = ocp.ocp_delete_device(
        &device_name,
    );

    println!("{}", res.unwrap());
}
