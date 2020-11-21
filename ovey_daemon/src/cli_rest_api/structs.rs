//! Route paths/urls for the REST-API for Ovey CLI.

use ovey_coordinator::data::{VirtualNetworkIdType, GuidIdType};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use crate::cli_rest_api::validation::{validate_device_name, validate_parent_device_name, validate_guid};

/// Payload for the REST interface of ovey daemon to create a device in both: coordinator and kernel
#[derive(Serialize, Deserialize, Debug, Builder, Default)]
#[builder(setter(into), build_fn(validate = "Self::validate"))]
pub struct CreateDeviceInput {
    network_id: VirtualNetworkIdType,
    virt_guid: GuidIdType,
    device_name: String,
    parent_device_name: String,
}

impl CreateDeviceInput {
    pub fn network_id(&self) -> &VirtualNetworkIdType {
        &self.network_id
    }
    pub fn virt_guid(&self) -> &GuidIdType {
        &self.virt_guid
    }
    pub fn device_name(&self) -> &str {
        &self.device_name
    }
    pub fn parent_device_name(&self) -> &str {
        &self.parent_device_name
    }
}

// comes from code generation
// https://docs.rs/derive_builder/0.9.0/derive_builder/
impl CreateDeviceInputBuilder {
    /// Constructs a new instance from an existing with an intermediate step
    /// via the builder. Advantage: If this comes from REST (deserialized)
    /// we can validate it easily again by constructing a new version of it.
    pub fn rebuild(input: CreateDeviceInput) -> Result<CreateDeviceInput, String> {
        // construct a new instance with the builder as middleware -> validation :)
        CreateDeviceInputBuilder::default()
            .virt_guid(input.virt_guid.to_owned())
            .device_name(input.device_name.to_owned())
            .parent_device_name(input.parent_device_name.to_owned())
            .network_id(input.network_id.to_owned())
            .build()
    }

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

/// Payload for the REST interface of ovey daemon to delete a device in both: the kernel and the coordinator.
#[derive(Serialize, Deserialize, Debug, Builder, Default)]
#[builder(setter(into), build_fn(validate = "Self::validate"))]
pub struct DeleteDeviceInput {
    // network_id: VirtualNetworkIdType,
    // virt_guid: VirtualGuidType,
    device_name: String,
}

impl DeleteDeviceInput {
    /*pub fn network_id(&self) -> VirtualNetworkIdType {
        self.network_id
    }*/
    /* pub fn virt_guid(&self) -> &str {
        &self.virt_guid
    }*/

    pub fn device_name(&self) -> &str {
        &self.device_name
    }
}

// comes from code generation
// https://docs.rs/derive_builder/0.9.0/derive_builder/
impl DeleteDeviceInputBuilder {
    /// Constructs a new instance from an existing with an intermediate step
    /// via the builder. Advantage: If this comes from REST (deserialized)
    /// we can validate it easily again by constructing a new version of it.
    pub fn rebuild(input: DeleteDeviceInput) -> Result<DeleteDeviceInput, String> {
        // construct a new instance with the builder as middleware -> validation :)
        DeleteDeviceInputBuilder::default()
            // .virt_guid(input.virt_guid.to_owned())
            // .network_id(input.network_id.to_owned())
            .device_name(input.device_name.to_owned())
            .build()
    }

    fn validate(&self) -> Result<(), String> {
        // it's okay to skip none values: that they are set is checked anyways by the builder
        if let Some(ref val) = self.device_name {
            validate_device_name(val)?;
        }
        /*if let Some(ref val) = self.virt_guid {
            validate_guid(val)?;
        }*/
        Ok(())
    }
}

/// This DTO tells the CLI the state of the deletion after a delete request.
#[derive(Serialize, Deserialize, Debug, Builder, Default)]
pub struct DeletionStateDto {
    deletion_local_successfully: bool,
    deletion_local_info_msg: Option<String>,
    deletion_coordinator_successfully: bool,
    deletion_coordinator_info_msg: Option<String>,
}

impl DeletionStateDto {
    pub fn deletion_local_successfully(&self) -> bool {
        self.deletion_local_successfully
    }
    pub fn deletion_local_info_msg(&self) -> Option<&String> {
        self.deletion_local_info_msg.as_ref()
    }
    pub fn deletion_coordinator_successfully(&self) -> bool {
        self.deletion_coordinator_successfully
    }
    pub fn deletion_coordinator_info_msg(&self) -> Option<&String> {
        self.deletion_coordinator_info_msg.as_ref()
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use uuid::Uuid;

    #[test]
    fn builder_works() {
        // comes from code generation
        // see https://crates.io/crates/derive_builder
        let foo = CreateDeviceInputBuilder::default()
            .virt_guid("dead:beef:0bad:f00d")
            .device_name("ovey0")
            .parent_device_name("rxe0")
            .network_id(Uuid::parse_str("c929e96d-6285-4528-b98e-b364d64790ae").unwrap())
            .build()
            .unwrap();
        println!("{:#?}", foo);

        let _new = CreateDeviceInputBuilder::rebuild(foo).unwrap();

        let foo = DeleteDeviceInputBuilder::default()
            .device_name("ovey0")
            // .network_id(Uuid::parse_str("c929e96d-6285-4528-b98e-b364d64790ae").unwrap())
            .build()
            .unwrap();
        println!("{:#?}", foo);

        let _new = DeleteDeviceInputBuilder::rebuild(foo).unwrap();
    }

}

