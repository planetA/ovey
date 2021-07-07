//! Route paths/urls for the REST-API for Ovey Daemon.

use uuid::Uuid;
use liboveyutil::types::GuidString;

/// {network} is uuid of the network.
pub const ROUTE_ADD_DEVICE_URL: &str  = "/network/{network}/device";
/// {network} is uuid of the network.
pub const ROUTE_LEASE_GID_URL: &str  = "/network/{network}/gid";
pub const ROUTE_RESOLVE_GID_URL: &str  = "/network/{network}/resolve_gid";
/// Endpoint where the Ovey daemon can ask for all devices in the current network.
pub const ROUTE_NETWORK_URL: &str     = "/network/{network}";
/// Under this endpoint a device can be managed. GET, DELETE, ...
/// {network} is uuid of the network, {device_id} the virtual guid string.
pub const ROUTE_DEVICE_URL: &str      = "/network/{network}/device/{device_id}";


/// Builds the endpoint starting with / inside Ovey coordinator REST Service,
/// where new devices can be created.
#[allow(dead_code)]
pub fn build_add_device_url(network_id: Uuid) -> String {
    ROUTE_ADD_DEVICE_URL.replace("{network}", &network_id.to_string())
}

/// Builds the endpoint starting with / inside Ovey coordinator REST Service,
/// where new devices can be created.
#[allow(dead_code)]
pub fn build_lease_gid_url(network_id: Uuid) -> String {
    ROUTE_LEASE_GID_URL.replace("{network}", &network_id.to_string())
}

/// Builds the endpoint starting with / inside Ovey coordinator REST Service,
/// where new devices can be created.
#[allow(dead_code)]
pub fn build_resolve_gid_url(network_id: Uuid) -> String {
    ROUTE_RESOLVE_GID_URL.replace("{network}", &network_id.to_string())
}

/// Builds the endpoint starting with / inside Ovey coordinator REST Service,
/// where a network can be queried with all its devices.
#[allow(dead_code)]
pub fn build_network_url(network_id: Uuid) -> String {
    ROUTE_NETWORK_URL.replace("{network}", &network_id.to_string())
}

/// Builds the endpoint url do modify a device.
#[allow(dead_code)]
pub fn build_device_url(network_id: Uuid, device_id: GuidString) -> String {
    ROUTE_DEVICE_URL
        .replace("{network}", &network_id.to_string())
        .replace("{device_id}", &device_id)
}
