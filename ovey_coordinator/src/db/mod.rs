//! "Database" (RAM HashMap) and service layer.

use std::sync::Mutex;
use std::collections::HashMap;
use crate::data::{DBType, VirtualNetworkIdType, VirtualizedDevice, VirtualGuidType};
use crate::rest::{VirtualizedDeviceDTO, VirtualizedDeviceInput};

lazy_static::lazy_static! {
    pub static ref DB: Mutex<DBType> = Mutex::new(HashMap::new());
}

pub fn get_all_data() -> HashMap<VirtualNetworkIdType, Vec<VirtualizedDeviceDTO>> {
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
pub fn add_device_to_network(network_id: &VirtualNetworkIdType, dev: VirtualizedDeviceInput) -> Result<VirtualizedDeviceDTO, String> {
    let id = &dev.virtual_device_guid_string().to_owned();
    {
        let mut db = DB.lock().unwrap();
        if !db.contains_key(&network_id) {
            return Err(format!("Network with id {} doesn't exists!", network_id));
        }

        let network = db.get_mut(&network_id).unwrap();
        if network.contains_key(dev.virtual_device_guid_string()) {
            return Err(format!("Virtual device with guid {} is already registered in virtual network {}!", dev.virtual_device_guid_string(), network_id));
        }

        network.insert(dev.virtual_device_guid_string().to_owned(), VirtualizedDevice::new(dev));
    }

    // release lock before next call
    let dto = get_device(network_id, id).unwrap();

    Ok(dto)
}
