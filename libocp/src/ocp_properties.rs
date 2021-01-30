//! This module describes all constants and operations related to our *Ovey Control Protocol* (OCP).
//! OCP includes all data that is transferred via generic netlink between the user component and
//! the linux kernel module. Please refer to "ocp-properties.h" which acts as the main spec.

use std::fmt;

/// The name of the netlink family we want to connect with.
pub const FAMILY_NAME: &str = "rdma-ovey";

pub const DEVICE_NAME_PATTERN: &str = "^ovey[0-9]+$";
pub const PARENT_DEVICE_NAME_PATTERN: &str = "^[A-z]+[0-9]+$";

// Implements the necessary trait for the "neli" lib on an enum called "OveyOperation".
// OveyOperation corresponds to "enum OveyOperation" in kernel module C code.
// Describes what callback function shall be invoked in the linux kernel module.

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
    DaemonBye => 7,
    DebugInitiateRequest => 8,
    ResolveCompletion => 9,
    DebugResolveAllCompletions => 10,
    KernelModuleBye => 11,
    StoreVirtPropertyPortLid => 12
);
impl neli::consts::genl::Cmd for OveyOperation {}
impl fmt::Display for OveyOperation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let numeric_value: u8 = u8::from(self);
        match self {
            OveyOperation::Unspec => write!(f, "OveyOperation::Unspec({})", numeric_value),
            OveyOperation::Echo => write!(f, "OveyOperation::Echo({})", numeric_value),
            OveyOperation::CreateDevice => write!(f, "OveyOperation::CreateDevice({})", numeric_value),
            OveyOperation::DeleteDevice => write!(f, "OveyOperation::DeleteDevice({})", numeric_value),
            OveyOperation::DebugRespondError => write!(f, "OveyOperation::DebugRespondError({})", numeric_value),
            OveyOperation::DeviceInfo => write!(f, "OveyOperation::DeviceInfo({})", numeric_value),
            OveyOperation::DaemonHello => write!(f, "OveyOperation::DaemonHello({})", numeric_value),
            OveyOperation::DaemonBye => write!(f, "OveyOperation::DaemonBye({})", numeric_value),
            OveyOperation::DebugInitiateRequest => write!(f, "OveyOperation::DebugInitiateRequest({})", numeric_value),
            OveyOperation::ResolveCompletion => write!(f, "OveyOperation::ResolveCompletion({})", numeric_value),
            OveyOperation::DebugResolveAllCompletions => write!(f, "OveyOperation::DebugResolveAllCompletions({})", numeric_value),
            OveyOperation::KernelModuleBye => write!(f, "OveyOperation::KernelModuleBye({})", numeric_value),
            OveyOperation::StoreVirtPropertyPortLid => write!(f, "OveyOperation::StoreVirtPropertyPortLid({})", numeric_value),
            _ =>  write!(f, "OveyOperation::<unknown>({})", numeric_value),
        }
    }
}

// Implements the necessary trait for the "neli" lib on an enum called "OveyAttribute".
// Command corresponds to "enum OveyAttribute" in kernel module C code.
// Describes the value type to data mappings inside the generic netlink packet payload.
neli::impl_var!( // also impls copy
    pub OveyAttribute,
    u16,
    Unspec => 0,
    Msg => 1,
    DeviceName => 2,
    ParentDeviceName => 3,
    NodeGuid => 4,
    ParentNodeGuid => 5,
    VirtNetUuidStr => 6,
    SocketKind => 7,
    CompletionId => 8,
    VirtPropertyU32 => 9,
    RealPropertyU32 => 10
);
impl neli::consts::genl::NlAttrType for OveyAttribute {}
impl fmt::Display for OveyAttribute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let numeric_value = u16::from(self);
        match self {
            OveyAttribute::Unspec => write!(f, "OveyAttribute::Unspec({})", numeric_value),
            OveyAttribute::Msg => write!(f, "OveyAttribute::Msg({})", numeric_value),
            OveyAttribute::DeviceName => write!(f, "OveyAttribute::DeviceName({})", numeric_value),
            OveyAttribute::ParentDeviceName => write!(f, "OveyAttribute::ParentDeviceName({})", numeric_value),
            OveyAttribute::NodeGuid => write!(f, "OveyAttribute::NodeGuid({})", numeric_value),
            OveyAttribute::ParentNodeGuid => write!(f, "OveyAttribute::ParentNodeGuid({})", numeric_value),
            OveyAttribute::VirtNetUuidStr => write!(f, "OveyAttribute::VirtNetUuidStr({})", numeric_value),
            OveyAttribute::SocketKind => write!(f, "OveyAttribute::SocketKind({})", numeric_value),
            OveyAttribute::CompletionId => write!(f, "OveyAttribute::CompletionId({})", numeric_value),
            OveyAttribute::VirtPropertyU32 => write!(f, "OveyAttribute::VirtPropertyU32({})", numeric_value),
            OveyAttribute::RealPropertyU32 => write!(f, "OveyAttribute::RealPropertyU32({})", numeric_value),
            _ => write!(f, "OveyAttribute::<unknown>({})", numeric_value),
        }
    }
}

// Used to identify the socket inside a process with multiple sockets.
// Also used as attribute value.
neli::impl_var!( // also impls copy
    pub OcpSocketKind,
    u32,
    // Socket used for daemon initiated requests and kernel replies
    DaemonInitiatedRequestsSocket => 0,
    // Socket used for kernel initiated requests and ovey userland replies
    KernelInitiatedRequestsSocket => 1
);


