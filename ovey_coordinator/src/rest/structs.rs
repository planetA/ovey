//! Data structures (Data transfer objects - DTOs) for the coordinator REST interface.
//! *Input: For Data Input
//! *DTO:   For Data Output
//! In many cases probably almost the same.

use serde::{Deserialize, Serialize};
use liboveyutil::endianness::Endianness;
use crate::data::{VirtualizedDevice, VirtualNetworkIdType, GuidIdType};
use derive_builder::Builder;
use std::collections::HashMap;
use liboveyutil::guid;
use uuid::Uuid;

/// This is the data for the REST-API that is expected as payload of a REST-Request
/// when a new file should be created.
#[derive(Serialize, Deserialize, Debug, Builder)]
#[builder(setter(into))]
pub struct VirtualizedDeviceInput {
    // name: especially helpful during development
    virtual_device_guid_string: String,
    // name: especially helpful during development
    physical_device_guid_string: String,
    /// device name, e.g. ovey0
    device_name: String,
    /// parent device name, e.g. rxe0 or mlx0
    parent_device_name: String,

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

    pub fn device_name(&self) -> &str {
        &self.device_name
    }
    pub fn parent_device_name(&self) -> &str {
        &self.parent_device_name
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
    /// device name, e.g. ovey0
    device_name: String,
    /// parent device name, e.g. rxe0 or mlx0
    parent_device_name: String,
}

impl VirtualizedDeviceDTO {
    pub fn new(entity: &VirtualizedDevice) -> Self {
        let virtual_device_guid_string = guid::guid_be_to_string(entity.virtual_guid_be());
        let virtual_device_guid_be = entity.virtual_guid_be();
        let virtual_device_guid_le = Endianness::change(entity.virtual_guid_be());
        let physical_device_guid_string = guid::guid_be_to_string(entity.physical_guid_be());
        let physical_device_guid_be = entity.physical_guid_be();
        let physical_device_guid_le = Endianness::change(entity.physical_guid_be());
        let device_name = entity.virtual_device_name().to_owned();
        let parent_device_name = entity.physical_device_name().to_owned();

        Self {
            virtual_device_guid_string,
            virtual_device_guid_be,
            virtual_device_guid_le,
            physical_device_guid_string,
            physical_device_guid_be,
            physical_device_guid_le,
            device_name,
            parent_device_name
        }
    }


    pub fn virtual_device_guid_string(&self) -> &str {
        &self.virtual_device_guid_string
    }
    pub fn virtual_device_guid_be(&self) -> u64 {
        self.virtual_device_guid_be
    }
    pub fn virtual_device_guid_le(&self) -> u64 {
        self.virtual_device_guid_le
    }
    pub fn physical_device_guid_string(&self) -> &str {
        &self.physical_device_guid_string
    }
    pub fn physical_device_guid_be(&self) -> u64 {
        self.physical_device_guid_be
    }
    pub fn physical_device_guid_le(&self) -> u64 {
        self.physical_device_guid_le
    }
    pub fn device_name(&self) -> &str {
        &self.device_name
    }
    pub fn parent_device_name(&self) -> &str {
        &self.parent_device_name
    }
}

/// This DTO exports all networks with all registered devices.
pub type AllNetworksDtoType = HashMap<VirtualNetworkIdType, Vec<VirtualizedDeviceDTO>>;

#[derive(Serialize, Deserialize, Debug)]
pub struct InitDataConfiguration {
    networks: HashMap<Uuid, Vec<GuidIdType>>
}

impl InitDataConfiguration {
    pub fn networks(&self) -> &HashMap<Uuid, Vec<String>> {
        &self.networks
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn builder_works() {
        // see https://crates.io/crates/derive_builder
        let foo = VirtualizedDeviceInputBuilder::default()
            .virtual_device_guid_string("1000:0000:0000:0000")
            .physical_device_guid_string("3000:0000:0000:0000")
            .parent_device_name("rxe0")
            .device_name("ovey0")
            .build()
            .unwrap();
        println!("{:#?}", foo);
    }

}
