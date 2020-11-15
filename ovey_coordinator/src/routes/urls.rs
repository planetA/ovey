//! Route paths/urls.

/// Returns the configuration of the coordinator.
pub const ROUTE_GET_CONFIG_URL: &str = "/config";
/// {network} is uuid of the network.
pub const ROUTE_POST_ADD_DEVICE_URL: &str = "/network/{network}/device";
