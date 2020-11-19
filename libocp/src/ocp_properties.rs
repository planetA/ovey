//! This module describes all constants and operations related to our *Ovey Control Protocol* (OCP).
//! OCP includes all data that is transferred via generic netlink between the user component and
//! the linux kernel module. Please refer to "ocp-properties.h" which acts as the main spec.

use neli::consts::{Cmd, NlAttrType};
use std::fmt;

/// The name of the netlink family we want to connect with.
pub const FAMILY_NAME: &str = "rdma-ovey";

// Implements the necessary trait for the "neli" lib on an enum called "OveyOperation".
// OveyOperation corresponds to "enum OveyOperation" in kernel module C code.
// Describes what callback function shall be invoked in the linux kernel module.
neli::impl_var_trait!(
    OveyOperation, u8, Cmd,
    Unspec => 0,
    Echo => 1,
    CreateDevice => 2,
    DeleteDevice => 3,
    DebugRespondError => 4,
    DeviceInfo => 5
);
impl Copy for OveyOperation {}
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
            OveyOperation::DeviceInfo => write!(f, "OveyOperation::DeviceInfo({})", numeric_value),
            _ =>  write!(f, "OveyOperation::<unknown>({})", numeric_value),
        }
    }
}

// Implements the necessary trait for the "neli" lib on an enum called "OveyAttribute".
// Command corresponds to "enum OveyAttribute" in kernel module C code.
// Describes the value type to data mappings inside the generic netlink packet payload.
neli::impl_var_trait!(
    OveyAttribute, u16, NlAttrType,
    Unspec => 0,
    Msg => 1,
    DeviceName => 2,
    ParentDeviceName => 3,
    NodeGuid => 4,
    VirtNetUuidStr => 5
);
impl Copy for OveyAttribute {}
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
            OveyAttribute::VirtNetUuidStr => write!(f, "OveyAttribute::VirtNetUuidStr({})", numeric_value),
            _ =>  write!(f, "OveyAttribute::<unknown>({})", numeric_value),
        }
    }
}

pub const DEVICE_NAME_PATTERN: &str = "ovey[0-9]+";
pub const PARENT_DEVICE_NAME_PATTERN: &str = "[A-z]+[0-9]+";
