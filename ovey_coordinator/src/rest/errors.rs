use actix_web::{
    dev::HttpResponseBuilder, error, http::header, http::StatusCode, HttpResponse,
};
use derive_more::Display;
use liboveyutil::types::{Uuid, VirtualNetworkIdType};

#[derive(Debug, Display)]
pub enum CoordinatorRestError {
    // 5XX errors


    // 4XX Errors

    /// Means that the allow-list from the init configuration doesn't cover the given network id.
    #[display(fmt = "The given virtual network '{}' is not supported.", _0)]
    VirtNetworkNotSupported(Uuid),
    /// Means that the allow-list from the init configuration doesn't cover the given virtual device guid.
    #[display(fmt = "The given virtual guid '{}' is not supported for the given virtual network '{}'.", _1, _0)]
    VirtDeviceGuidNotSupported(Uuid, String),
    /// Means that a virtual device is already registered for the given virtual network id.
    #[display(fmt = "The given virtual guid '{}' is already registered in the virtual network '{}'.", _1, _0)]
    VirtDeviceAlreadyRegistered(Uuid, String),
    /// Means that the virtual device is supported by the coordinator but not yet registered/activated.
    #[display(fmt = "The virtual device with guid '{}' is not registered in the virtual network '{}'.", _1, _0)]
    VirtDeviceNotYetRegistered(Uuid, String),
    #[display(fmt = "The device name '{}' is already registered in this virtual network ({}) for a device with another guid.", name, network)]
    VirtDeviceNameAlreadyRegistered{network: VirtualNetworkIdType, name: String}

}

// IDE tells that Display is not implemented for CoordinatorRestError, but it gets implemented
// during compile time by derive_more
impl error::ResponseError for CoordinatorRestError {
    fn status_code(&self) -> StatusCode {
        match *self {
            // 5XX Errors

            // 4XX errors
            CoordinatorRestError::VirtNetworkNotSupported(_) => StatusCode::NOT_FOUND,
            CoordinatorRestError::VirtDeviceGuidNotSupported(_, _) => StatusCode::NOT_FOUND,
            CoordinatorRestError::VirtDeviceAlreadyRegistered(_, _) => StatusCode::CONFLICT,
            CoordinatorRestError::VirtDeviceNameAlreadyRegistered{network: _, name: _} => StatusCode::CONFLICT,
            CoordinatorRestError::VirtDeviceNotYetRegistered(_, _) => StatusCode::NOT_FOUND,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponseBuilder::new(self.status_code())
            .set_header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .body(self.to_string())
    }
}
