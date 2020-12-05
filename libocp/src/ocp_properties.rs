//! This module describes all constants and operations related to our *Ovey Control Protocol* (OCP).
//! OCP includes all data that is transferred via generic netlink between the user component and
//! the linux kernel module. Please refer to "ocp-properties.h" which acts as the main spec.

use std::fmt;

/// The name of the netlink family we want to connect with.
pub const FAMILY_NAME: &str = "rdma-ovey";

pub const DEVICE_NAME_PATTERN: &str = "ovey[0-9]+";
pub const PARENT_DEVICE_NAME_PATTERN: &str = "[A-z]+[0-9]+";

// Implements the necessary trait for the "neli" lib on an enum called "OveyOperation".
// OveyOperation corresponds to "enum OveyOperation" in kernel module C code.
// Describes what callback function shall be invoked in the linux kernel module.

impl_trait!(
    /// Trait marking constants valid for use in
    /// [`Genlmsghdr`][crate::genl::Genlmsghdr] field, `cmd`.
    pub Cmd,
    u8,
    /// Wrapper valid for use with all values in the [`Genlmsghdr`]
    /// field, `cmd`
    OveyOperationWrapper,
    OveyOperation
);
// TODO this is strange.. wait for https://github.com/jbaublitz/neli/issues/99
impl neli::consts::genl::Cmd for OveyOperation {}
neli::impl_var!( // also impls copy
    pub OveyOperation,
    u8,
    Unspec => 0,
    Echo => 1,
    CreateDevice => 2,
    DeleteDevice => 3,
    DebugRespondError => 4,
    DeviceInfo => 5,
    DaemonHello => 6,
    DaemonBye => 7
);
impl fmt::Display for OveyOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // weird hack but otherwise I can't get the numeric value of the enum -.-
        // this doesn't work: https://stackoverflow.com/questions/31358826/how-do-i-convert-an-enum-reference-to-a-number
        let numeric_value: u8 = unsafe { std::mem::transmute_copy(self) };
        match self {
            OveyOperation::Unspec => write!(f, "OveyOperation::Unspec({})", numeric_value),
            OveyOperation::Echo => write!(f, "OveyOperation::Echo({})", numeric_value),
            OveyOperation::CreateDevice => write!(f, "OveyOperation::CreateDevice({})", numeric_value),
            OveyOperation::DeleteDevice => write!(f, "OveyOperation::DeleteDevice({})", numeric_value),
            OveyOperation::DebugRespondError => write!(f, "OveyOperation::DebugRespondError({})", numeric_value),
            OveyOperation::DeviceInfo => write!(f, "OveyOperation::DeviceInfo({})", numeric_value),
            OveyOperation::DaemonHello => write!(f, "OveyOperation::DaemonHello({})", numeric_value),
            OveyOperation::DaemonBye => write!(f, "OveyOperation::DaemonBye({})", numeric_value),
            _ =>  write!(f, "OveyOperation::<unknown>({})", numeric_value),
        }
    }
}

// Implements the necessary trait for the "neli" lib on an enum called "OveyAttribute".
// Command corresponds to "enum OveyAttribute" in kernel module C code.
// Describes the value type to data mappings inside the generic netlink packet payload.
impl_trait!(
    /// Marker trait for types usable in the
    /// [`Nlattr`][crate::genl::Nlattr] field, `nla_type`
    pub NlAttrType,
    u16,
    /// Wrapper that is usable with all values in the
    /// [`Nlattr`][crate::genl::Nlattr] field, `nla_type`.
    pub OveyAttributeWrapper,
    OveyAttribute
);
// TODO this is strange.. wait for https://github.com/jbaublitz/neli/issues/99
impl neli::consts::genl::NlAttrType for OveyAttribute {}
neli::impl_var!( // also impls copy
    pub OveyAttribute,
    u16,
    Unspec => 0,
    Msg => 1,
    DeviceName => 2,
    ParentDeviceName => 3,
    NodeGuid => 4,
    ParentNodeGuid => 5,
    VirtNetUuidStr => 6
);
impl fmt::Display for OveyAttribute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // weird hack but otherwise I can't get the numeric value of the enum -.-
        // this doesn't work: https://stackoverflow.com/questions/31358826/how-do-i-convert-an-enum-reference-to-a-number
        let numeric_value: u8 = unsafe { std::mem::transmute_copy(self) };
        match self {
            OveyAttribute::Unspec => write!(f, "OveyAttribute::Unspec({})", numeric_value),
            OveyAttribute::Msg => write!(f, "OveyAttribute::Msg({})", numeric_value),
            OveyAttribute::DeviceName => write!(f, "OveyAttribute::DeviceName({})", numeric_value),
            OveyAttribute::ParentDeviceName => write!(f, "OveyAttribute::ParentDeviceName({})", numeric_value),
            OveyAttribute::NodeGuid => write!(f, "OveyAttribute::NodeGuid({})", numeric_value),
            OveyAttribute::ParentNodeGuid => write!(f, "OveyAttribute::ParentNodeGuid({})", numeric_value),
            OveyAttribute::VirtNetUuidStr => write!(f, "OveyAttribute::VirtNetUuidStr({})", numeric_value),
            _ =>  write!(f, "OveyAttribute::<unknown>({})", numeric_value),
        }
    }
}
