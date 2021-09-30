use actix_web::{
    dev::HttpResponseBuilder, error, http::header, http::StatusCode, HttpResponse,
};
use derive_more::Display;
use uuid::Uuid;
use liboveyutil::types::Gid;

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
    #[display(fmt = "Gid '{}' not found.", _0)]
    GidNotFound(Gid),
    #[display(fmt = "LID '{}' not found.", _0)]
    LidNotFound(u32),
    #[display(fmt = "QP '{}' not found.", _0)]
    QpNotFound(u32),
    #[display(fmt = "Real or virtual gid is not unique.")]
    GidConflict,
    #[display(fmt = "Attempt to store reserved address.")]
    GidReserved,
    #[display(fmt = "Addresses resolve to conflicting devices.")]
    DeviceConflict,
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
            CoordinatorRestError::LidNotFound(..) => StatusCode::NOT_FOUND,
            CoordinatorRestError::QpNotFound(..) => StatusCode::NOT_FOUND,
            CoordinatorRestError::GidConflict => StatusCode::CONFLICT,
            CoordinatorRestError::GidReserved => StatusCode::CONFLICT,
            CoordinatorRestError::DeviceConflict => StatusCode::CONFLICT,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponseBuilder::new(self.status_code())
            .set_header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .body(self.to_string())
    }
}
