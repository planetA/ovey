//! Route paths/urls for the REST-API for Ovey Daemon.

use crate::data::{VirtualNetworkIdType, VirtualGuidType};

/// Returns the configuration of the coordinator.
pub const ROUTE_GET_CONFIG_URL: &str = "/config";
/// {network} is uuid of the network.
pub const ROUTE_POST_ADD_DEVICE_URL: &str = "/network/{network}/device";
/// {network} is uuid of the network, {device_id} the virtual guid string.
pub const ROUTE_GET_DEVICE_INFO: &str = "/network/{network}/device/{device_id}";

pub fn build_route_post_add_device_url(network_id: VirtualNetworkIdType) -> String {
    ROUTE_POST_ADD_DEVICE_URL.replace("{network}", &network_id.to_string())
}

pub fn build_route_get_device_info_url(network_id: VirtualNetworkIdType, device_id: VirtualGuidType) -> String {
    ROUTE_POST_ADD_DEVICE_URL
        .replace("{network}", &network_id.to_string())
        .replace("{device_id}", &device_id)
}
