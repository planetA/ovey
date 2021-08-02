use actix_web::{
    dev::HttpResponseBuilder, error, http::header, http::StatusCode, HttpResponse,
};
use derive_more::Display;
use uuid::Uuid;

#[derive(Debug, Display)]
pub enum CoordinatorRestError {
    // 5XX errors


    // 4XX Errors
    #[display(fmt = "Network '{}' not found.", _0)]
    NetworkUuidNotFound(Uuid),
    #[display(fmt = "Device '{}' not found in the network '{}'.", _1, _0)]
    DeviceUuidNotFound(Uuid, Uuid),
    #[display(fmt = "Port '{}' not found inside device '{}'.", _1, _0)]
    PortNotFound(Uuid, u16),
    #[display(fmt = "Gid '{:08x}:{:08x}' not found.", _0, _1)]
    GidNotFound(u64, u64),
    #[display(fmt = "Real or virtual gid is not unique.")]
    GidConflict,
}

// IDE tells that Display is not implemented for CoordinatorRestError, but it gets implemented
// during compile time by derive_more
impl error::ResponseError for CoordinatorRestError {
    fn status_code(&self) -> StatusCode {
        match *self {
            // 5XX Errors

            // 4XX errors
            CoordinatorRestError::NetworkUuidNotFound(..) => StatusCode::NOT_FOUND,
            CoordinatorRestError::DeviceUuidNotFound(..) => StatusCode::NOT_FOUND,
            CoordinatorRestError::PortNotFound(..) => StatusCode::NOT_FOUND,
            CoordinatorRestError::GidNotFound(..) => StatusCode::NOT_FOUND,
            CoordinatorRestError::GidConflict => StatusCode::CONFLICT,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponseBuilder::new(self.status_code())
            .set_header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .body(self.to_string())
    }
}
