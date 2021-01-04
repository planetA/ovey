//! Demo that uses OCP against the Ovey Kernel module to test some functionality.
//! Make sure that this version of ocp_properties matches the one inside the kernel module!

use libocp::ocp_core::{Ocp, OcpError};

/// Demo for get info command.
fn main() {
    let ocp = Ocp::connect().unwrap();

    let device_name = "ovey0".to_string();

    for i in 0..100 {
        println!("#{}", i);
        let res = ocp.ocp_get_device_info(
            &device_name,
        );

        match res {
            Ok(res) => {
                println!("{}", res)
            }
            Err(err) => {
                match err {
                    OcpError::DeviceDoesntExist => {
                        println!("Device ovey0 doesn't exist in the kernel.")
                    }
                    err => {
                        panic!("Received unexected OcpError! {:#?}", err)
                    }
                }
            }
        }
    }

}
