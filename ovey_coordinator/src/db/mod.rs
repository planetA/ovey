//! "Database" (RAM HashMap) and service layer.

use std::sync::Mutex;
use std::collections::HashMap;
use crate::data::{DBType, VirtualizedDevice};
use crate::rest::structs::{VirtualizedDeviceDTO, VirtualizedDeviceInput, AllNetworksDtoType};
use crate::rest::errors::CoordinatorRestError;
use crate::data::VirtualizedNetworkDataType;
use liboveyutil::types::{VirtualNetworkIdType, GuidString, Uuid};

lazy_static::lazy_static! {
    pub static ref DB: Mutex<DBType> = Mutex::new(HashMap::new());
}

/// Returns the whole database with all networks and their current registered devices.
pub fn db_get_all_data() -> AllNetworksDtoType {
    let data = &*DB.lock().unwrap();
    let mut all_data = HashMap::new();
    data.keys().for_each(|network_key| {
        let mut dtos = Vec::new();
        data.get(network_key).unwrap().keys().for_each(| virtual_guid| {
            dtos.push(
                // NO: in this case dead lock because this function holds the lock
                // get_device(network_key, virtual_guid).unwrap()
                VirtualizedDeviceDTO::new(
                    data.get(network_key).unwrap()
                        .get(virtual_guid).unwrap()
                )
            )
        });
        all_data.insert(network_key.to_owned(), dtos);
    });

    all_data
}

/// Returns info about all devices in a specific network.
pub fn db_get_network_data(network_id: &VirtualNetworkIdType) -> Result<Vec<VirtualizedDeviceDTO>, CoordinatorRestError> {
    let network = &*DB.lock().unwrap();
    let network = network.get(network_id);
    let network = network.ok_or(CoordinatorRestError::VirtNetworkNotSupported(network_id.to_owned()))?;
    let devices = network.values()
        .map(|dev| VirtualizedDeviceDTO::new(dev))
        .collect::<Vec<VirtualizedDeviceDTO>>();

    Ok(devices)
}


/// Returns the data for a single device
pub fn db_get_device_data(network_id: &VirtualNetworkIdType, dev_id: &GuidString)
                          -> Result<VirtualizedDeviceDTO, CoordinatorRestError> {
    let network = &*DB.lock().unwrap();
    let network = network.get(network_id);
    let network = network.ok_or(CoordinatorRestError::VirtNetworkNotSupported(network_id.to_owned()))?;
    let device = network.get(dev_id).ok_or(CoordinatorRestError::VirtDeviceNotYetRegistered(network_id.to_owned(), dev_id.to_owned()));
    let device = device.map(|d| VirtualizedDeviceDTO::new(d));
    device
}

/*pub fn get_device(network_id: &VirtualNetworkIdType, dev: &VirtualGuidType) -> Option<VirtualizedDeviceDTO> {
    let db = DB.lock().unwrap();
    let network_data = db.get(&network_id)?;
    let dev = network_data.get(dev)?;
    Some(VirtualizedDeviceDTO::new(dev))
}*/

/// Adds a virtual network entry to the db. This means that the coordinator can
/// manage devices of that network.
pub fn db_register_network(id: VirtualNetworkIdType) -> Result<(), String> {
    let mut db = DB.lock().unwrap();
    if db.contains_key(&id) {
        Err(format!("Network with id {} already exists!", id))
    } else {
        db.insert(id, HashMap::new());
        Ok(())
    }
}

/// Assigns a virtualized rdma device to a virtualized network.
pub fn db_add_device_to_network(network_id: &VirtualNetworkIdType, dev: VirtualizedDeviceInput) -> Result<VirtualizedDeviceDTO, CoordinatorRestError> {
    // validate first
    check_device_is_allowed(network_id, &dev.virtual_device_guid_string().to_owned())?;

    let id = &dev.virtual_device_guid_string().to_owned();
    {
        let mut db = DB.lock().unwrap();
        if !db.contains_key(&network_id) {
            // should never happen; on program init all keys are created
            return Err(
                CoordinatorRestError::VirtNetworkNotSupported(network_id.to_owned())
            );
        }

        let network = db.get_mut(&network_id).unwrap();
        if network.contains_key(dev.virtual_device_guid_string()) {
            return Err(
                CoordinatorRestError::VirtDeviceAlreadyRegistered(
                    network_id.to_owned(),
                    dev.virtual_device_guid_string().to_owned()
                )
            );
        }

        let key = dev.virtual_device_guid_string().to_owned();
        let entity = VirtualizedDevice::new(dev);
        network.insert(key, entity);
    }

    // release lock before next call
    let dto = db_get_device_data(network_id, id).unwrap();

    Ok(dto)
}

/// Returns the old device as DTO on success, otherwise error.
pub fn db_delete_device_from_network(network_id: &VirtualNetworkIdType, dev_id: &GuidString) -> Result<VirtualizedDeviceDTO, CoordinatorRestError> {
    let mut network = DB.lock().unwrap();
    let network = network.get_mut(network_id);
    let network = network.ok_or(CoordinatorRestError::VirtNetworkNotSupported(network_id.to_owned()))?;
    let dto = network.remove(dev_id);
    let dto = dto.map(|e| VirtualizedDeviceDTO::new(&e));
    let dto = dto.ok_or(CoordinatorRestError::VirtDeviceNotYetRegistered(network_id.to_owned(), dev_id.to_owned()));
    dto
}

/// Checks against the coordinator config if the specified device is allowed inside the specified network.
/// (if this coordinator is responsible for them)
pub fn check_device_is_allowed(network_id: &Uuid, device_id: &GuidString) -> Result<(), CoordinatorRestError> {
    // validate device guid
    let devs = crate::config::CONFIG.networks().get(network_id);
    if devs.is_none() {
        return Err(CoordinatorRestError::VirtNetworkNotSupported(network_id.to_owned()));
    }

    let devs = devs.unwrap();
    if !devs.contains(device_id) {
        Err(CoordinatorRestError::VirtDeviceGuidNotSupported(network_id.to_owned(), device_id.to_owned()))
    } else {
        Ok(())
    }
}
