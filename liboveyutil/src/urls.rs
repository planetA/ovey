//! Route paths/urls for the REST-API for Ovey Daemon.

use uuid::Uuid;

/// Endpoint to all known networks
pub const ROUTE_NETWORK_ALL: &str = "/networks";
/// Endpoint to a known network identified by UUID
pub const ROUTE_NETWORK_ONE: &str = "/networks/{network}";
/// Endpoint to all devices in a known network.
pub const ROUTE_DEVICES_ALL: &str = "/networks/{network}/devices";
/// Endpoint to all devices in a known network. Both are identified by UUID.
pub const ROUTE_DEVICES_ONE: &str = "/networks/{network}/devices/{device}";
/// Endpoint to all GUIDs of a device. A device may have multiple GUIDs, if it
/// has multiple ports, for example.
pub const ROUTE_GUIDS_DEVICE: &str = "/networks/{network}/devices/{device}/guids";
/// Endpoint to all GIDs of a device. A device may have multiple GIDs.
pub const ROUTE_GIDS_DEVICE: &str = "/networks/{network}/devices/{device}/gids";
/// Endpoint to all GIDs in a network. A device may have multiple GIDs.
pub const ROUTE_GIDS_ALL: &str = "/networks/{network}/gids";

/// Builds the endpoint starting with / inside Ovey coordinator REST Service,
/// where a network can be queried with all its devices.
pub fn build_network_url(endpoint: &str, network: Uuid) -> String {
    endpoint.replace("{network}", &network.to_string())
}

/// Builds the endpoint url do modify a device.
pub fn build_device_url(endpoint: &str, network: Uuid, device: Uuid) -> String {
    endpoint
        .replace("{network}", &network.to_string())
        .replace("{device}", &device.to_string())
}
