//! Service-layer for the communication with Ovey coordinator.

use ovey_coordinator::rest::structs::{VirtualizedDeviceDTO, VirtualizedDeviceInput, VirtualizedDeviceInputBuilder, InitDataConfiguration as CoordinatorInitDataConfiguration, AllNetworksDtoType};
use ovey_daemon::structs::{CreateDeviceInput};
use crate::config::CONFIG;
use ovey_coordinator::OVEY_COORDINATOR_PORT;
use liboveyutil::types::{VirtualNetworkIdType, GuidString};

// TODO remove?
/*/// This function can be used to find if a device with a specific device name (e.g. ovey0)
/// has already been registered. In this case it returns the guid of this element.
/// If this returns None than a new device with the given ID can be created.
fn find_device_guid_by_name_in_list(list: &Vec<VirtualizedDeviceDTO>, dev_name: &str) -> Option<GuidIdType> {
    let vec = list.iter()
        .filter(|dto| dto.device_name() == dev_name)
        .map(|dto| dto.virtual_device_guid_string())
        .collect::<Vec<&str>>();
    let guid = if vec.is_empty() { None } else { Some((vec[0]).to_owned()) };
    guid
}*/

pub async fn check_config_and_environment() -> std::io::Result<()> {
    let mut actual_up = 0;
    let expected_up = CONFIG.coordinators().len();

    if !CONFIG.check_coordinators() {
        return Ok(());
    }

    if expected_up == 0 {
        return Err(std::io::Error::new(std::io::ErrorKind::Other,
            "There is not a single Ovey coordinator configured."));
    }

    // check all hosts are available
    for (network, host) in CONFIG.coordinators() {
        let mut host = host.to_owned();
        // e.g. http://localhost:13337
        host.push_str(&format!(":{}", OVEY_COORDINATOR_PORT));

        let resp = reqwest::get(&host).await;
        let resp = resp.map_err(|_| std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Ovey coordinator on configured host '{}' IS NOT UP.", &host)))?;
        let resp = resp.json::<AllNetworksDtoType>().await;
        let json = resp.map_err(|_| std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Ovey coordinator @ host '{}' sent invalid response.", &host)))?;

        if json.contains_key(network) {
            actual_up += 1;
        } else {
            error!(
                "Ovey coordinator on configured host '{}' does not know about network '{}'!",
                &host,
                network
            );
        }
    }

    return if actual_up != expected_up {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "WARNING: Not all Ovey coordinators are available."))
    } else {
        debug!("INFO: All Ovey coordinators are available.");
        Ok(())
    }
}
