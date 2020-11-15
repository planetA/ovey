//! "Database" (RAM HashMap) and service layer.

use std::sync::Mutex;
use std::collections::HashMap;
use crate::data::{DBType, VirtualNetworkIdType, VirtualizedDevice, VirtualGuidType};
use crate::rest::structs::{VirtualizedDeviceDTO, VirtualizedDeviceInput, AllNetworksDtoType};
use crate::rest::errors::CoordinatorRestError;
use uuid::Uuid;

lazy_static::lazy_static! {
    pub static ref DB: Mutex<DBType> = Mutex::new(HashMap::new());
}

pub fn get_all_data() -> AllNetworksDtoType {
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

pub fn get_device(network_id: &VirtualNetworkIdType, dev: &VirtualGuidType) -> Option<VirtualizedDeviceDTO> {
    let db = DB.lock().unwrap();
    let network_data = db.get(&network_id)?;
    let dev = network_data.get(dev)?;
    Some(VirtualizedDeviceDTO::new(dev))
}

/// Adds a virtual network entry to the db. This means that the coordinator can
/// manage devices of that network.
pub fn register_network(id: VirtualNetworkIdType) -> Result<(), String> {
    let mut db = DB.lock().unwrap();
    if db.contains_key(&id) {
        Err(format!("Network with id {} already exists!", id))
    } else {
        db.insert(id, HashMap::new());
        Ok(())
    }
}

/// Assigns a virtualized rdma device to a virtualized network.
pub fn add_device_to_network(network_id: &VirtualNetworkIdType, dev: VirtualizedDeviceInput) -> Result<VirtualizedDeviceDTO, CoordinatorRestError> {
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

        network.insert(dev.virtual_device_guid_string().to_owned(), VirtualizedDevice::new(dev));
    }

    // release lock before next call
    let dto = get_device(network_id, id).unwrap();

    Ok(dto)
}

pub fn check_device_is_allowed(network_id: &Uuid, device_id: &VirtualGuidType) -> Result<(), CoordinatorRestError> {
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