//! Demo that uses OCP against the Ovey Kernel module to test some functionality.
//! Make sure that this version of ocp_properties matches the one inside the kernel module!

use libocp::ocp_core::{Ocp};

/// Demo for create command.
fn main() {
    let mut ocp = Ocp::connect(1).unwrap();

    let device_name = "ovey0".to_string();
    let parent_device_name = "rxe0".to_string();
    let network_uuid_str = "c929e96d-6285-4528-b98e-b364d64790ae".to_string();

    let node_guid_he = 0xdead_beef_0bad_f00d_u64;

    let res = ocp.ocp_create_device(
                                &device_name,
                                &parent_device_name,
                                node_guid_he,
                                &network_uuid_str
    );

    println!("{}", res.unwrap());
}
