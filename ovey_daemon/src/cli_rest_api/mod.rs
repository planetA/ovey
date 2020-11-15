//! Route paths/urls for the REST-API for Ovey CLI.

use ovey_coordinator::data::{VirtualNetworkIdType, VirtualGuidType};
use ovey_coordinator::OVEY_COORDINATOR_PORT;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use crate::cli_rest_api::validation::{validate_device_name, validate_parent_device_name, validate_guid};

pub const OVEY_DAEMON_PORT: usize = OVEY_COORDINATOR_PORT + 1;

pub mod validation;
pub mod errors;

/// A POST-Request to this URL creates an Ovey device inside the kernel.
pub const ROUTE_CREATE_DEVICE: &str = "/device";
/// A DELETE-Request to this URL delete an Ovey device inside the kernel.
pub const ROUTE_DELETE_DEVICE: &str = "/device";
/// A GET-Request to this URL tests an OCP Echo Message (to test OCP netlink communication).
pub const ROUTE_KERNEL_ECHO: &str = "/kernel/echo";

#[derive(Serialize, Deserialize, Debug, Builder, Default)]
#[builder(setter(into), build_fn(validate = "Self::validate"))]
pub struct DeviceInput {
    network: VirtualNetworkIdType,
    virt_guid: VirtualGuidType,
    device_name: String,
    parent_device_name: String
}

impl DeviceInput {

    /// Constructs a new instance from an existing with an intermediate step
    /// via the builder. Advantage: If this comes from REST (deserialized)
    /// we can validate it easily again by constructing a new version of it.
    pub fn new(di: &DeviceInput) -> Self {
       // construct a new instance with the builder as middleware -> validation :)
        DeviceInputBuilder::default()
            .virt_guid(di.virt_guid.to_owned())
            .device_name(di.device_name.to_owned())
            .parent_device_name(di.parent_device_name.to_owned())
            .network(di.network.to_owned())
            .build()
            .unwrap()
    }
}

// comes from code generation
// https://docs.rs/derive_builder/0.9.0/derive_builder/
impl DeviceInputBuilder {
    fn validate(&self) -> Result<(), String> {
        // it's okay to skip none values: that they are set is checked anyways by the builder
        if let Some(ref val) = self.device_name {
            validate_device_name(val)?;
        }
        if let Some(ref val) = self.parent_device_name {
            validate_parent_device_name(val)?;
        }
        if let Some(ref val) = self.virt_guid {
            validate_guid(val)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn builder_works() {
        // comes from code generation
        // see https://crates.io/crates/derive_builder
        let foo = DeviceInputBuilder::default()
            .virt_guid("dead:beef:0bad:f00d")
            .device_name("ovey0")
            .parent_device_name("rxe0")
            .network(Uuid::parse_str("c929e96d-6285-4528-b98e-b364d64790ae").unwrap())
            .build()
            .unwrap();
        println!("{:#?}", foo);

        let new = DeviceInput::new(&foo);
    }

}

