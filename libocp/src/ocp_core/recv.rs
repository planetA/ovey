use crate::ocp_properties::OveyAttribute;
use std::fmt::{Display, Formatter};
use std::fmt;
use liboveyutil::guid::guid_u64_to_string;
use liboveyutil::lid::lid_u16_to_string;
use crate::ocp_core::ocp::{OveyGenNetlMsgType};
use neli::Nl;
use liboveyutil::types::{GuidInternalType, GuidString, LidString};

/// Struct that holds all the data that can be received via OCP from the kernel. It's up
/// to the caller function to extract the right data.
/// We derive Serialize and Deserialize because it's useful to pass this via REST
/// for debugging. Also, additional compile time overhead is negligible.
#[derive(Debug)]
pub struct OCPRecData {
    msg: Option<String>,
    device_name: Option<String>,
    parent_device_name: Option<String>,
    node_guid: Option<u64>,
    node_lid: Option<u16>,
    parent_node_guid: Option<u64>,
    virt_network_uuid_str: Option<String>,
    socket_kind: Option<u32>,
    completion_id: Option<u64>,
}

impl OCPRecData {
    /// Creates a new OCPRecData struct. It parses each attribute that neli received
    /// via generic netlink to its proper Rust runtime type. This is ONLY NECESSARY
    /// for attributes we want to receive.
    pub fn new(res: &OveyGenNetlMsgType) -> Self {
        let mut msg = None;
        let mut device_name = None;
        let mut parent_device_name = None;
        let mut node_guid = None;
        let mut node_lid = None;
        let mut parent_node_guid = None;
        let mut virt_network_uuid_str = None;
        let mut socket_kind = None;
        let mut completion_id = None;

        let payload = res.get_payload().unwrap();

        payload.get_attr_handle().iter().for_each(|attr| {
            match attr.nla_type {
                OveyAttribute::Msg => {
                    msg.replace(String::deserialize(attr.nla_payload.as_ref()).unwrap());
                },
                OveyAttribute::DeviceName => {
                    device_name.replace(String::deserialize(attr.nla_payload.as_ref()).unwrap());
                },
                OveyAttribute::ParentDeviceName => {
                    parent_device_name.replace(String::deserialize(attr.nla_payload.as_ref()).unwrap());
                },
                OveyAttribute::VirtNetUuidStr => {
                    virt_network_uuid_str.replace(String::deserialize(attr.nla_payload.as_ref()).unwrap());
                },
                OveyAttribute::NodeGuid => {
                    node_guid.replace(u64::deserialize(attr.nla_payload.as_ref()).unwrap());
                },
                OveyAttribute::NodeLid => {
                    node_lid.replace(u16::deserialize(attr.nla_payload.as_ref()).unwrap());
                },
                OveyAttribute::ParentNodeGuid => {
                    parent_node_guid.replace(u64::deserialize(attr.nla_payload.as_ref()).unwrap());
                },
                OveyAttribute::SocketKind => {
                    socket_kind.replace(u32::deserialize(attr.nla_payload.as_ref()).unwrap());
                }
                OveyAttribute::CompletionId => {
                    completion_id.replace(u64::deserialize(attr.nla_payload.as_ref()).unwrap());
                }
                OveyAttribute::UnrecognizedVariant(_) => {panic!("Received UnrecognizedVariant")}
                OveyAttribute::Unspec => { panic!("Received unspec") }

                // they will never appear in this struct
                OveyAttribute::VirtPropertyU32 => {}
                OveyAttribute::RealPropertyU32 => {}
            }
        });

        OCPRecData {
            msg,
            device_name,
            parent_device_name,
            // we receive it in big endian format from be;
            // restore host endian format
            node_guid: node_guid,
            node_lid: node_lid,
            parent_node_guid: parent_node_guid,
            virt_network_uuid_str,
            socket_kind,
            completion_id,
        }
    }

    /// Getter for [`msg`]
    pub fn msg(&self) -> Option<&String> {
        self.msg.as_ref()
    }
    /// Getter for [`device_name`]
    pub fn device_name(&self) -> Option<&String> {
        self.device_name.as_ref()
    }
    /// Getter for [`parent_device_name`]
    pub fn parent_device_name(&self) -> Option<&String> {
        self.parent_device_name.as_ref()
    }
    /// Getter for [`virt_network_uuid_str`]
    pub fn virt_network_uuid_str(&self) -> Option<&String> {
        self.virt_network_uuid_str.as_ref()
    }
    /// Getter for [`node_guid`]
    pub fn node_guid(&self) -> Option<GuidInternalType> {
        self.node_guid
    }
    /// Getter for [`node_guid`]
    pub fn node_lid(&self) -> Option<u16> {
        self.node_lid
    }
    /// Getter for [`parent_node_guid`]
    pub fn parent_node_guid(&self) -> Option<GuidInternalType> {
        self.parent_node_guid
    }
    /// Getter for [`msg`]
    pub fn node_guid_str(&self) -> Option<GuidString> {
        self.node_guid.map(|val| guid_u64_to_string(val))
    }
    pub fn node_lid_str(&self) -> Option<LidString> {
        self.node_lid.map(|val| lid_u16_to_string(val))
    }
    /// Getter for [`msg`]
    pub fn parent_node_guid_str(&self) -> Option<GuidString> {
        self.parent_node_guid.map(|val| guid_u64_to_string(val))
    }
    /// Getter for [`socket_kind`]
    pub fn socket_kind(&self) -> Option<u32> {
        self.socket_kind
    }
    /// Getter for [`completion_id`]
    pub fn completion_id(&self) -> Option<u64> {
        self.completion_id
    }
}

impl Display for OCPRecData {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "OCPRecData {{\n\
            \x20   msg: {:?}\n\
            \x20   device_name: {:?}\n\
            \x20   parent_device_name: {:?}\n\
            \x20   guid: {:?}\n\
            \x20   |- guid_string: {:?}\n\
            \x20   parent_guid: {:?}\n\
            \x20   |- parent_guid_string: {:?}\n\
            \x20   virt_network_uuid_str: {:?}\n\
            \x20   socket_kind: {:?}\n\
            \x20   CompletionId: {:?}\n\
        }}",
               self.msg,
               self.device_name,
               self.parent_device_name,
               self.node_guid,
               self.node_guid_str(),
               self.parent_node_guid,
               self.parent_node_guid_str(),
               self.virt_network_uuid_str,
               self.socket_kind,
               self.completion_id
        )
    }
}
