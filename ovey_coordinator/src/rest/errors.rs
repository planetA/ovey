use actix_web::{
    dev::HttpResponseBuilder, error, http::header, http::StatusCode, HttpResponse,
};

// use derive_more::{Display, Error};
use serde::export::Formatter;
use uuid::Uuid;

#[derive(Debug)]
pub enum CoordinatorRestError {
    /// Means that the allow-list from the init configuration doesn't cover the given network id.
    VirtNetworkNotSupported(Uuid),
    /// Means that the allow-list from the init configuration doesn't cover the given virtual device guid.
    VirtDeviceGuidNotSupported(Uuid, String),
    /// Means that a virtual device is already registered for the given virtual network id.
    VirtDeviceAlreadyRegistered(Uuid, String)
}

impl std::fmt::Display for CoordinatorRestError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            CoordinatorRestError::VirtNetworkNotSupported(ref id) => {
                write!(f, "The given virtual network '{}' is not supported.", id)
            },
            CoordinatorRestError::VirtDeviceGuidNotSupported(ref network_id, ref device_id) => {
                write!(f, "The given virtual guid '{}' is not supported for the given virtual network '{}'.", device_id, network_id)
            }
            CoordinatorRestError::VirtDeviceAlreadyRegistered(ref network_id, ref device_id) => {
                write!(f, "The given virtual guid '{}' is already registered in the virtual network '{}'.", device_id, network_id)
            }
        }
    }
}

impl error::ResponseError for CoordinatorRestError {
    fn status_code(&self) -> StatusCode {
        match *self {
            CoordinatorRestError::VirtNetworkNotSupported(_) => StatusCode::BAD_REQUEST,
            CoordinatorRestError::VirtDeviceGuidNotSupported(_, _) => StatusCode::BAD_REQUEST,
            CoordinatorRestError::VirtDeviceAlreadyRegistered(_, _) => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponseBuilder::new(self.status_code())
            .set_header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .body(self.to_string())
    }
}
