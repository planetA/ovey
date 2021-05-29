//! Demo that uses OCP against the Ovey Kernel module to test some functionality.
//! Make sure that this version of ocp_properties matches the one inside the kernel module!

use libocp::ocp_core::{Ocp};

/// Demo that:
/// - creates Ovey device
/// - gets information
/// - deletes it
fn main() {
    let ocp = Ocp::connect().unwrap();

    println!("creates, queries, and deletes 1000 ovey devices");

    let parent_device_name = "rxe0".to_string();
    let network_uuid_str = "c929e96d-6285-4528-b98e-b364d64790ae".to_string();
    // "dead:beef:0bad:f00d" => 1004492983682117086
    let node_guid = 0xdead_beef_0bad_f00d_u64;
    let node_lid = 0xdead_u16;

    for i in 1..1000 {
        let device_name = format!("ovey{}", i);
        println!("#{:>3}: creating device ovey{}", i, i);

        let _res = ocp.ocp_create_device(
            &device_name,
            &parent_device_name,
            node_guid,
            node_lid,
            &network_uuid_str
        ).expect("Must be created!");
    }

    for i in 1..1000 {
        let device_name = format!("ovey{}", i);

        println!("#{:>3}:Fetched device info from OCP", i);
        let _res = ocp.ocp_get_device_info(
            &device_name,
        ).unwrap();
    }

    for i in 1..1000 {
        let device_name = format!("ovey{}", i);

        println!("#{:>3}:deleting device ovey{}", i, i);
        let _res = ocp.ocp_delete_device(
            &device_name,
        ).expect("must be deleted");
    }

}
