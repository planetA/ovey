//! This bundles all sub modules necessary for OCP.

#[macro_use]
// I don't know why this is necessary.. this is legacy I thought..
// since macros can be called like a module path
extern crate neli;

use crate::ocp_core::{OCPRecData, build_nl_attr, Ocp};
use crate::ocp_properties::{OveyOperation, OveyAttribute};

pub mod ocp_core;
pub mod ocp_properties;

/// Convenient wrapper function that creates an
/// new Ovey device inside the Ovey Kernel Module
/// via OCP. It returns whether the operation was
/// successfully or not.
pub fn ocp_create_device(ga: &mut Ocp,
                         device_name: &str,
                         parent_device_name: &str,
                         node_guid_be: u64,
                         network_uuid_str: &str,
                         ) -> Result<OCPRecData, String> {
    ga.send_and_ack(
        OveyOperation::CreateDevice,
        vec![
            build_nl_attr(OveyAttribute::DeviceName, device_name),
            build_nl_attr(OveyAttribute::ParentDeviceName, parent_device_name),
            build_nl_attr(OveyAttribute::NodeGuid, node_guid_be),
            build_nl_attr(OveyAttribute::VirtNetUuidStr, network_uuid_str),
        ]
    )
}

/// Convenient wrapper function that deletes a n
/// Ovey device inside the Ovey Kernel Module
/// via OCP. It returns whether the operation was
/// successfully or not.
pub fn ocp_delete_device(ga: &mut Ocp,
                         device_name: &str
                         ) -> Result<OCPRecData, String> {
    ga.send_and_ack(
        OveyOperation::DeleteDevice,
        vec![
            build_nl_attr(OveyAttribute::DeviceName, device_name)
        ]
    )
}

/// Convenient wrapper function that gets info about an
/// Ovey device inside the Ovey Kernel Module
/// via OCP. It returns whether the operation was
/// successfully or not.
pub fn ocp_get_device_info(ga: &mut Ocp,
                           device_name: &str
                           ) -> Result<OCPRecData, String> {
    ga.send_and_ack(
        OveyOperation::DeviceInfo,
        vec![
            build_nl_attr(OveyAttribute::DeviceName, device_name)
        ]
    )
}

/// Convenient wrapper function that tests OCP
/// with the Kernel Module by sending an ECHO
/// request. Kernel should reply with an
/// message with the proper content.
pub fn ocp_echo(ga: &mut Ocp,
                echo_msg: &str
                ) -> Result<OCPRecData, String> {
    ga.send_single_and_ack(
        OveyOperation::Echo,
        OveyAttribute::Msg,
        echo_msg
    )
}

/// Convenient wrapper function that triggers a
/// error response via OCP by the Ovey Kernel Module.
pub fn ocp_debug_respond_error(ga: &mut Ocp) -> Result<OCPRecData, String> {
    ga.send_and_ack(
        OveyOperation::DebugRespondError,
        vec![]
    )
}
