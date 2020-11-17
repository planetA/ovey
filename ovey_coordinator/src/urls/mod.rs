//! Route paths/urls for the REST-API for Ovey Daemon.

use crate::data::{VirtualNetworkIdType, VirtualGuidType};

/// Returns the configuration of the coordinator.
pub const ROUTE_GET_CONFIG_URL: &str    = "/config";
/// {network} is uuid of the network.
pub const ROUTE_ADD_DEVICE_URL: &str   = "/network/{network}/device";
/// Under this endpoint a device can be managed. GET, DELETE, ...
/// {network} is uuid of the network, {device_id} the virtual guid string.
pub const ROUTE_DEVICE: &str = "/network/{network}/device/{device_id}";

pub fn build_add_device_url(network_id: VirtualNetworkIdType) -> String {
    ROUTE_ADD_DEVICE_URL.replace("{network}", &network_id.to_string())
}

/// Builds the endpoint url do modify a device.
pub fn build_device_url(network_id: VirtualNetworkIdType, device_id: VirtualGuidType) -> String {
    ROUTE_DEVICE
        .replace("{network}", &network_id.to_string())
        .replace("{device_id}", &device_id)
}
