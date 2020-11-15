//! Data structures for the coordinator database.

use uuid::Uuid;
use std::collections::HashMap;
use crate::rest::structs::VirtualizedDeviceInput;

/// A guid is a big endian encoded u64.
pub type GuidType = u64;
/// Virtual GUID as String (e.g. dead:beef:affe:cafe) is the key.
/// This is easier to read/write during development and overhead is neglible.
pub type VirtualGuidType = String;
/// Virtual networks are identified by an UUID.
pub type VirtualNetworkIdType = Uuid;
/// Virtualized networks are a map from virtual guid of the device to the virtualized data of that device.
pub type VirtualizedNetworkDataType = HashMap<VirtualGuidType, VirtualizedDevice>;
/// The key of our database is the virtual network id. Our database can hold data
/// for multiple virtual networks.
/// Our database is a hashmap from virtual network id to the virtual network data.
pub type DBType = HashMap<VirtualNetworkIdType, VirtualizedNetworkDataType>;

#[derive(Debug)]
pub struct VirtualizedDevice {
    // name: especially helpful during development
    virtual_device_name: Option<String>,
    /// Virtual GUID in big endian format.
    virtual_guid_be: GuidType,
    // name: especially helpful during development
    physical_device_name: Option<String>,
    /// Physical GUID in big endian format.
    physical_guid_be: GuidType,
    qp_num: u64,
    // add more properties that needs to be virtualized..
}

impl VirtualizedDevice {
    pub fn virtual_device_name(&self) -> &Option<String> {
        &self.virtual_device_name
    }
    pub fn virtual_guid_be(&self) -> u64 {
        self.virtual_guid_be
    }
    pub fn physical_device_name(&self) -> &Option<String> {
        &self.physical_device_name
    }
    pub fn physical_guid_be(&self) -> u64 {
        self.physical_guid_be
    }
    pub fn qp_num(&self) -> u64 {
        self.qp_num
    }

    pub fn new(input: VirtualizedDeviceInput) -> Self {
        Self {
            virtual_device_name: None,
            physical_device_name: None,
            virtual_guid_be: librdmautil::guid_string_to_ube64(input.virtual_device_guid_string()),
            physical_guid_be: librdmautil::guid_string_to_ube64(input.physical_device_guid_string()),
            qp_num: 0,
        }
    }
}




