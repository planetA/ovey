//! Demo that uses OCP against the Ovey Kernel module to test some functionality.
//! Make sure that this version of ocp_properties matches the one inside the kernel module!

use libocp::ocp_core::{Ocp};
use libocp::ocp_get_device_info;

/// Demo for get info command.
fn main() {
    let mut ocp = Ocp::connect(1).unwrap();

    let device_name = "ovey0".to_string();

    let res = ocp_get_device_info(
        &mut ocp,
        &device_name,
    );

    println!("{}", res.unwrap());
}
