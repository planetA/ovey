//! Demo that uses OCP against the Ovey Kernel module to test some functionality.
//! Make sure that this version of ocp_properties matches the one inside the kernel module!

use libocp::ocp_core::{Ocp};

/// Demo for create command.
fn main() {
    let mut ocp = Ocp::connect(1, false).unwrap();

    let device_name = "ovey0".to_string();
    let parent_device_name = "rxe0".to_string();
    let network_uuid_str = "c929e96d-6285-4528-b98e-b364d64790ae".to_string();
    // "dead:beef:0bad:f00d" => 1004492983682117086
    let guid_be = 1004492983682117086_u64;

    let res = ocp.ocp_create_device(
                                &device_name,
                                &parent_device_name,
                                guid_be,
                                &network_uuid_str
    );

    println!("{}", res.unwrap());
}
