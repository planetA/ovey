//! Service-layer for the communication with Ovey coordinator.

use ovey_coordinator::rest::structs::{VirtualizedDeviceDTO, VirtualizedDeviceInput, VirtualizedDeviceInputBuilder, InitDataConfiguration as CoordinatorInitDataConfiguration};
use ovey_daemon::errors::DaemonRestError;
use ovey_daemon::structs::{CreateDeviceInput, CreateDeviceInputBuilder, DeleteDeviceInput, DeleteDeviceInputBuilder};
use crate::config::CONFIG;
use ovey_coordinator::OVEY_COORDINATOR_PORT;
use actix_web::http::StatusCode;
use liboveyutil::guid;
use liboveyutil::types::{VirtualNetworkIdType, GuidIdType};

fn get_host(network_id: &VirtualNetworkIdType) -> Result<String, DaemonRestError> {
    // http://localhost or http://123.56.78.1 or https://foo.bar
    let host = CONFIG.coordinators().get(network_id);
    let host = host.ok_or(DaemonRestError::UnknownNetwork(network_id.to_owned()))?;
    let port = OVEY_COORDINATOR_PORT;

    let url = format!("{}:{}", host, port);

    Ok(url)
}

/// Queries the virtual network for the specified device name. This way a device can be found by it's unique
/// device name. Return type is Result<Option<> because:
/// - http errors can occur (Result)
/// - a device name may be taken or not; both is fine (Option)
pub async fn rest_lookup_device_guid_by_name(network_id: &VirtualNetworkIdType, dev_name: &str) -> Result<Option<GuidIdType>, DaemonRestError> {
    let host = get_host(network_id)?;
    // endpoint inside REST service with starting /
    let endpoint = ovey_coordinator::urls::build_network_url(network_id.to_owned());
    let url = format!("{}{}", host, endpoint);

    let client = reqwest::Client::new();
    let res = client.get(&url)
        .send()
        .await;

    let res = res.map_err(|_| DaemonRestError::CoordinatorDoesntRespond(network_id.to_owned()))?;
    if res.status() == StatusCode::NOT_FOUND {
        return Err(DaemonRestError::UnknownNetwork(network_id.to_owned()));
    }
    let network_devices = res.json::<Vec<VirtualizedDeviceDTO>>().await;
    let network_devices = network_devices.map_err(|e| DaemonRestError::IllegalCoordinatorResponse)?;

    // now search in all devices of the network for the guid of the specified device name
    // because the coordinator makes sure that device name is unique per network we can assume here
    // as well, that there will only be one.
    let guid = find_device_guid_by_name_in_list(&network_devices, dev_name);

    Ok(guid)
}

/// Forwards the request from the CLI to create a device to the coordinator.
/// Returns the DTO from the coordinator on success.
pub async fn rest_forward_create_device(input: CreateDeviceInput, physical_guid_str: GuidIdType) -> Result<VirtualizedDeviceDTO, DaemonRestError> {
    let host = get_host(&input.network_id())?;
    // endpoint inside REST service with starting /
    let endpoint = ovey_coordinator::urls::build_add_device_url(input.network_id().to_owned());
    let url = format!("{}{}", host, endpoint);

    // Transform payload from cli request to payload for ovey coordinator
    let payload: Result<VirtualizedDeviceInput, String> = VirtualizedDeviceInputBuilder::default()
        .virtual_device_guid_string(input.virt_guid())
        .physical_device_guid_string(physical_guid_str)
        .parent_device_name(input.parent_device_name())
        .device_name(input.device_name())
        .build();
    let payload = payload.map_err(|e| DaemonRestError::MalformedPayload(e))?;

    let client = reqwest::Client::new();
    let res = client.post(&url)
        .json(&payload)
        .send()
        .await;

    let res = res.map_err(|_| DaemonRestError::CoordinatorDoesntRespond(input.network_id().to_owned()))?;

    if res.status() == StatusCode::NOT_FOUND {
        return Err(DaemonRestError::DeviceDoesntExist(
            input.virt_guid().to_owned(),
            input.network_id().to_owned())
        );
    }
    if res.status() == StatusCode::CONFLICT {
        return Err(DaemonRestError::DeviceAlreadyRegistered(
            input.virt_guid().to_owned(),
            input.network_id().to_owned())
        );
    }

    let res = res.json::<VirtualizedDeviceDTO>().await;
    let res = res.map_err(|_| DaemonRestError::IllegalCoordinatorResponse)?;
    Ok(res)
}

/// Forwards the request from the CLI to delete a device to the coordinator.
/// Returns the DTO from the coordinator on success.
pub async fn rest_forward_delete_device(device_id: &GuidIdType, network_id: &VirtualNetworkIdType) -> Result<VirtualizedDeviceDTO, DaemonRestError> {
    // http://localhost or http://123.56.78.1 or https://foo.bar
    let host = get_host(network_id)?;
    // endpoint inside REST service with starting /
    let endpoint = ovey_coordinator::urls::build_device_url(network_id.to_owned(), device_id.to_owned());
    let url = format!("{}{}", host, endpoint);

    let client = reqwest::Client::new();
    let res = client.delete(&url).send().await;
    let res = res.map_err(|_| DaemonRestError::CoordinatorDoesntRespond(network_id.to_owned()))?;

    if res.status() == StatusCode::NOT_FOUND {
        return Err(DaemonRestError::DeviceDoesntExist(
            device_id.to_owned(),
            network_id.to_owned())
        );
    }

    let res = res.json::<VirtualizedDeviceDTO>().await;
    let res = res.map_err(|_| DaemonRestError::IllegalCoordinatorResponse)?;
    Ok(res)
}

/// Checks if the coordinator allows the specific device in the specific network.
/// This is useful to check beforehand if a create device operation is allowed.
/// We fetch the data from /config endpoint from coordinator.
pub async fn rest_check_device_is_allowed(network_id: &VirtualNetworkIdType, guid_str: &GuidIdType) -> Result<bool, DaemonRestError> {
    let host = get_host(network_id)?;
    let endpoint = ovey_coordinator::urls::ROUTE_GET_CONFIG_URL;
    let url = format!("{}{}", host, endpoint);

    let client = reqwest::Client::new();
    let res = client.get(&url)
        .send()
        .await;

    let res = res.map_err(|_| DaemonRestError::CoordinatorDoesntRespond(network_id.to_owned()))?;
    let coordinator_cfg = res.json::<CoordinatorInitDataConfiguration>().await;
    let coordinator_cfg = coordinator_cfg.map_err(|e| DaemonRestError::MalformedPayload(e.to_string()))?;

    let network = coordinator_cfg.networks().get(network_id);
    let network = network.ok_or(DaemonRestError::UnknownNetwork(network_id.to_owned()))?;

    let found = network.iter().any(|guid| guid == guid_str);
    Ok(found)
}

/// This function can be used to find if a device with a specific device name (e.g. ovey0)
/// has already been registered. In this case it returns the guid of this element.
/// If this returns None than a new device with the given ID can be created.
fn find_device_guid_by_name_in_list(list: &Vec<VirtualizedDeviceDTO>, dev_name: &str) -> Option<GuidIdType> {
    let vec = list.iter()
        .filter(|dto| dto.device_name() == dev_name)
        .map(|dto| dto.virtual_device_guid_string())
        .collect::<Vec<&str>>();
    let guid = if vec.is_empty() { None } else { Some((vec[0]).to_owned()) };
    guid
}