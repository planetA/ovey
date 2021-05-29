//! Data structures for the coordinator database.

use std::collections::HashMap;
use crate::rest::structs::VirtualizedDeviceInput;
use liboveyutil::guid;
use liboveyutil::types::{GuidString, VirtualNetworkIdType, GuidInternalType};

/// Virtualized networks are a map from virtual guid of the device to the virtualized data of that device.
pub type VirtualizedNetworkDataType = HashMap<GuidString, VirtualizedDevice>;
/// The key of our database is the virtual network id. Our database can hold data
/// for multiple virtual networks.
/// Our database is a hashmap from virtual network id to the virtual network data.
pub type DBType = HashMap<VirtualNetworkIdType, VirtualizedNetworkDataType>;

#[derive(Debug)]
pub struct VirtualizedDevice {
    // e.g. "ovey0"
    virtual_device_name: String,
    /// Virtual GUID.
    virtual_guid: GuidInternalType,
    // e.g. "rxe0"
    physical_device_name: String,
    /// Physical GUID
    physical_guid: GuidInternalType,
    qp_num: u64,
    // add more properties that needs to be virtualized..
}

impl VirtualizedDevice {

    pub fn virtual_device_name(&self) -> &String {
        &self.virtual_device_name
    }
    pub fn virtual_guid(&self) -> u64 {
        self.virtual_guid
    }
    pub fn physical_device_name(&self) -> &String {
        &self.physical_device_name
    }
    pub fn physical_guid(&self) -> u64 {
        self.physical_guid
    }

    #[allow(dead_code)]
    pub fn qp_num(&self) -> u64 {
        self.qp_num
    }

    pub fn new(input: VirtualizedDeviceInput) -> Self {
        Self {
            virtual_device_name: input.device_name().to_owned(),
            physical_device_name: input.parent_device_name().to_owned(),
            virtual_guid: guid::guid_string_to_u64(input.virtual_device_guid_string()),
            physical_guid: guid::guid_string_to_u64(input.physical_device_guid_string()),
            qp_num: 0,
        }
    }
}




