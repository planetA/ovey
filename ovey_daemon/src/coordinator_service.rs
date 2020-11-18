//! Service-layer for the communication with Ovey coordinator.

use ovey_coordinator::rest::structs::{VirtualizedDeviceDTO, VirtualizedDeviceInput, VirtualizedDeviceInputBuilder};
use ovey_daemon::errors::DaemonRestError;
use ovey_daemon::structs::{CreateDeviceInput, CreateDeviceInputBuilder, DeleteDeviceInput, DeleteDeviceInputBuilder};
use crate::config::CONFIG;
use ovey_coordinator::OVEY_COORDINATOR_PORT;
use ovey_coordinator::data::VirtualNetworkIdType;
use actix_web::http::StatusCode;

fn get_host(network_id: &VirtualNetworkIdType) -> Result<String, DaemonRestError> {
    // http://localhost or http://123.56.78.1 or https://foo.bar
    let host = CONFIG.coordinators().get(network_id);
    let host = host.ok_or(DaemonRestError::UnknownNetwork(network_id.to_owned()))?;
    let port = OVEY_COORDINATOR_PORT;

    let url = format!("{}:{}", host, port);

    Ok(url)
}

/// Forwards the request from the CLI to create a device to the coordinator.
/// Returns the DTO from the coordinator on success.
pub async fn forward_create_device(input: CreateDeviceInput) -> Result<VirtualizedDeviceDTO, DaemonRestError> {
    // verify input
    let input: Result<CreateDeviceInput, String> = CreateDeviceInputBuilder::rebuild(input);
    let input = input.map_err(|e| DaemonRestError::MalformedPayload(e))?;

    let host = get_host(&input.network_id())?;
    // endpoint inside REST service with starting /
    let endpoint = ovey_coordinator::urls::build_add_device_url(input.network_id().to_owned());
    let url = format!("{}{}", host, endpoint);

    // Transform payload from cli request to payload for ovey coordinator
    let payload: Result<VirtualizedDeviceInput, String> = VirtualizedDeviceInputBuilder::default()
        .virtual_device_guid_string(input.virt_guid())
        // TODO get this from netlink
        .physical_device_guid_string("10D0:10D0:10D0:10D0:")
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
pub async fn forward_delete_device(input: DeleteDeviceInput) -> Result<VirtualizedDeviceDTO, DaemonRestError> {
    // verify input
    let input: Result<DeleteDeviceInput, String> = DeleteDeviceInputBuilder::rebuild(input);
    let input = input.map_err(|e| DaemonRestError::MalformedPayload(e))?;

    // http://localhost or http://123.56.78.1 or https://foo.bar
    let host = get_host(&input.network_id())?;
    // endpoint inside REST service with starting /
    let endpoint = ovey_coordinator::urls::build_device_url(input.network_id().to_owned(), input.virt_guid().to_owned());
    let url = format!("{}{}", host, endpoint);

    let client = reqwest::Client::new();
    let res = client.delete(&url).send().await;
    let res = res.map_err(|_| DaemonRestError::CoordinatorDoesntRespond(input.network_id().to_owned()))?;

    if res.status() == StatusCode::NOT_FOUND {
        return Err(DaemonRestError::DeviceDoesntExist(
            input.virt_guid().to_owned(),
            input.network_id().to_owned())
        );
    }

    let res = res.json::<VirtualizedDeviceDTO>().await;
    let res = res.map_err(|_| DaemonRestError::IllegalCoordinatorResponse)?;
    Ok(res)
}
