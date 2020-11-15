//! Data structures (Data transfer objects - DTOs) for the coordinator REST interface.
//! *Input: For Data Input
//! *DTO:   For Data Output
//! In many cases probably almost the same.

use serde::{Deserialize, Serialize};
use librdmautil::endianness::Endianness;
use crate::data::{VirtualizedDevice, VirtualNetworkIdType};
use derive_builder::Builder;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Builder)]
#[builder(setter(into))]
pub struct VirtualizedDeviceInput {
    // name: especially helpful during development
    virtual_device_guid_string: String,
    // name: especially helpful during development
    physical_device_guid_string: String,

    // qp_num: u64,
    // add more properties that needs to be virtualized..
}

impl VirtualizedDeviceInput {
    pub fn virtual_device_guid_string(&self) -> &str {
        &self.virtual_device_guid_string
    }
    pub fn physical_device_guid_string(&self) -> &str {
        &self.physical_device_guid_string
    }
}

/// The output for a virtualized device.
#[derive(Serialize, Deserialize, Debug)]
pub struct VirtualizedDeviceDTO {
    virtual_device_guid_string: String,
    virtual_device_guid_be: u64,
    virtual_device_guid_le: u64,
    physical_device_guid_string: String,
    physical_device_guid_be: u64,
    physical_device_guid_le: u64,
}

impl VirtualizedDeviceDTO {
    pub fn new(entity: &VirtualizedDevice) -> Self {
        let virtual_device_guid_string = librdmautil::guid_be_to_string(entity.virtual_guid_be());
        let virtual_device_guid_be = entity.virtual_guid_be();
        let virtual_device_guid_le = Endianness::change(entity.virtual_guid_be());
        let physical_device_guid_string = librdmautil::guid_be_to_string(entity.physical_guid_be());
        let physical_device_guid_be = entity.physical_guid_be();
        let physical_device_guid_le = Endianness::change(entity.physical_guid_be());

        Self {
            virtual_device_guid_string,
            virtual_device_guid_be,
            virtual_device_guid_le,
            physical_device_guid_string,
            physical_device_guid_be,
            physical_device_guid_le
        }
    }
}

/// This DTO exports all networks with all registered devices.
pub type AllNetworksDtoType = HashMap<VirtualNetworkIdType, Vec<VirtualizedDeviceDTO>>;

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn builder_works() {
        // see https://crates.io/crates/derive_builder
        let foo = VirtualizedDeviceInputBuilder::default()
            .virtual_device_guid_string("1000:0000:0000:0000")
            .physical_device_guid_string("3000:0000:0000:0000")
            .build()
            .unwrap();
        println!("{:#?}", foo);
    }

}
