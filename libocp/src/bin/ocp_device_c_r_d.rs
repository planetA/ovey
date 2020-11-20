//! Demo that uses OCP against the Ovey Kernel module to test some functionality.
//! Make sure that this version of ocp_properties matches the one inside the kernel module!

use libocp::ocp_core::{Ocp};

/// Demo that:
/// - creates Ovey device
/// - gets information
/// - deletes it
fn main() {
    let mut ocp = Ocp::connect(1, true).unwrap();

    let device_name = "ovey0".to_string();
    let parent_device_name = "rxe0".to_string();
    let network_uuid_str = "c929e96d-6285-4528-b98e-b364d64790ae".to_string();
    // "dead:beef:0bad:f00d" => 1004492983682117086
    let guid_be = 1004492983682117086_u64;


    println!("Fetched device info from OCP");
    let exists = ocp.ocp_get_device_info(&device_name);
    if exists.is_ok() {
        println!("Device exists! Delete it");
        let _ = ocp.ocp_delete_device(&device_name).expect("delete must work");
    }


    println!("creating device ovey0");
    let _res = ocp.ocp_create_device(
        &device_name,
        &parent_device_name,
        guid_be,
        &network_uuid_str
    ).expect("Must be created!");


    println!("Fetched device info from OCP");
    let res = ocp.ocp_get_device_info(
        &device_name,
    ).expect("must get info");

    println!("{}", res);

    println!("deleting device ovey0");
    let _res = ocp.ocp_delete_device(
        &device_name,
    ).expect("must be deleted");

    println!("deleted device ovey0");
}
