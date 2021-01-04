use derive_more::Display;
use neli::err::NlError;

/// OCP Errors that may occur.
#[derive(Debug, Display)]
pub enum OcpError {
    /// Device already exists.
    DeviceAlreadyExist,
    /// Device doesn't exist.
    DeviceDoesntExist,
    /// The request returned invalid error code.
    #[display(fmt = "Invalid(err_code={})", _0)]
    Invalid(libc::c_int),
    /// The underlying netlink socket library could not parse the result.
    /// Its likely that after these errors the socket is in bad state and doesn't work longer.
    #[display(fmt = "LowLevelError(neli_error={})", _0)]
    LowLevelError(NlError),
}

// IDE may show that "Display" is not implemented, but it gets implemented during build time
impl std::error::Error for OcpError {}
