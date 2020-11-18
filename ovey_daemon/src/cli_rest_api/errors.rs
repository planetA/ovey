use actix_web::{
    dev::HttpResponseBuilder, error, http::header, http::StatusCode, HttpResponse,
};

use derive_more::{Display};
use ovey_coordinator::data::{VirtualNetworkIdType, VirtualGuidType};


#[derive(Debug, Display)]
pub enum DaemonRestError {
    /// Kernel doesn't respond. Can't create device.
    // #[display(fmt = "Validation error on field: {}", _0)]

    // 5XX Errors
    #[display(fmt = "The kernel doesn't respond. Can't create the device.")]
    KernelDoesntRespond,
    #[display(fmt = "The coordinator for the network '{}' doesn't respond. Can't verify request.", _0)]
    CoordinatorDoesntRespond(VirtualNetworkIdType),
    #[display(fmt = "The coordinator replied with an invalid response.")]
    IllegalCoordinatorResponse,

    // 4XX Errors
    #[display(fmt = "The given network '{}' is unknown/not configured for this Ovey daemon.", _0)]
    UnknownNetwork(VirtualNetworkIdType),
    #[display(fmt = "The device '{}' is already registered in the network {}", _0, _1)]
    DeviceAlreadyRegistered(VirtualGuidType, VirtualNetworkIdType),
    #[display(fmt = "The device '{}' can't be deleted because it doesn't exist in the network '{}'", _0, _1)]
    DeviceDoesntExist(VirtualGuidType, VirtualNetworkIdType),
    #[display(fmt = "The request payload is invalid because of: '{}'", _0)]
    MalformedPayload(String)
}

impl error::ResponseError for DaemonRestError {
    fn status_code(&self) -> StatusCode {
        match *self {
            // 5XX
            DaemonRestError::KernelDoesntRespond => StatusCode::SERVICE_UNAVAILABLE,
            DaemonRestError::CoordinatorDoesntRespond(_) => StatusCode::SERVICE_UNAVAILABLE,
            DaemonRestError::IllegalCoordinatorResponse => StatusCode::INTERNAL_SERVER_ERROR,

            // 4XX
            DaemonRestError::UnknownNetwork(_) => StatusCode::NOT_FOUND,
            DaemonRestError::DeviceAlreadyRegistered(_, _) => StatusCode::CONFLICT,
            DaemonRestError::DeviceDoesntExist(_, _) => StatusCode::NOT_FOUND,
            DaemonRestError::MalformedPayload(_) => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponseBuilder::new(self.status_code())
            .set_header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .body(self.to_string())
    }
}
