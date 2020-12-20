use actix_web::{
    dev::HttpResponseBuilder, error, http::header, http::StatusCode, HttpResponse,
};

use derive_more::{Display};
use liboveyutil::types::{VirtualNetworkIdType, GuidIdType};


#[derive(Debug, Display, Clone)]
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
    #[display(fmt = "Ovey daemon can't connect to Ovey kernel module via OCP. Can't create device: {}", info)]
    OcpCantConnect{info: String},
    #[display(fmt = "Ovey daemon could not finish Ovey operation successfully. {}", info)]
    OcpOperationFailed{info: String},
    #[display(fmt = "OCP tells that the device '{}' doesn't exists on this machine.", info)]
    OcpDeviceNotFound{info: String},
    #[display(fmt = "OCP tells that the device '{}' already exists on this machine.", info)]
    OcpDeviceAlreadyExists{info: String},
    #[display(fmt = "An internal error occurred in Ovey daemon during your request. {}", info)]
    OtherInternalError{info: String},

    // 4XX Errors
    #[display(fmt = "The given network '{}' is unknown/not configured for this Ovey daemon.", _0)]
    UnknownNetwork(VirtualNetworkIdType),
    #[display(fmt = "The device '{}' is already registered in the network {}", _0, _1)]
    DeviceAlreadyRegistered(GuidIdType, VirtualNetworkIdType),
    #[display(fmt = "The device '{}' can't be deleted because it doesn't exist in the network '{}'", _0, _1)]
    DeviceDoesntExist(GuidIdType, VirtualNetworkIdType),
    #[display(fmt = "The request payload is invalid because of: '{}'", _0)]
    MalformedPayload(String),
    #[display(fmt = "The device with guid '{}' is not allowed inside the network '{}' according to the config of the coordinator!", virt_guid, network_id)]
    DeviceNotAllowed{network_id: VirtualNetworkIdType, virt_guid: GuidIdType }
}

// IDE says Display is not implemented but it gets implemented during compile time
impl error::ResponseError for DaemonRestError {
    fn status_code(&self) -> StatusCode {
        match *self {
            // 5XX
            DaemonRestError::KernelDoesntRespond => StatusCode::SERVICE_UNAVAILABLE,
            DaemonRestError::CoordinatorDoesntRespond(_) => StatusCode::SERVICE_UNAVAILABLE,
            DaemonRestError::IllegalCoordinatorResponse => StatusCode::INTERNAL_SERVER_ERROR,
            DaemonRestError::OcpCantConnect{info: _} => StatusCode::INTERNAL_SERVER_ERROR,
            DaemonRestError::OcpOperationFailed{info: _} => StatusCode::INTERNAL_SERVER_ERROR,
            DaemonRestError::OtherInternalError{info: _} => StatusCode::INTERNAL_SERVER_ERROR,

            // 4XX
            DaemonRestError::UnknownNetwork(_) => StatusCode::NOT_FOUND,
            DaemonRestError::DeviceAlreadyRegistered(_, _) => StatusCode::CONFLICT,
            DaemonRestError::DeviceDoesntExist(_, _) => StatusCode::NOT_FOUND,
            DaemonRestError::OcpDeviceNotFound{info: _} => StatusCode::NOT_FOUND,
            DaemonRestError::OcpDeviceAlreadyExists{info: _} => StatusCode::CONFLICT,
            DaemonRestError::DeviceNotAllowed{network_id: _, virt_guid: _} => StatusCode::BAD_REQUEST,
            DaemonRestError::MalformedPayload(_) => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponseBuilder::new(self.status_code())
            .set_header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .body(self.to_string())
    }
}
