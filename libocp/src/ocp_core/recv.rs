use crate::ocp_properties::OveyAttribute;
use liboveyutil::endianness::Endianness;
use std::fmt::{Display, Formatter};
use std::fmt;
use liboveyutil::guid::guid_u64_to_string;
use crate::ocp_core::ocp::{OveyGeNlMsg};

/// Struct that holds all the data that can be received via OCP from the kernel. It's up
/// to the caller function to extract the right data.
/// We derive Serialize and Deserialize because it's useful to pass this via REST
/// for debugging. Also, additional compile time overhead is negligible.
///
/// OCP takes care of guid endianness divergence between kernel and userland.
/// Inside the kernel module the guid is stored as big endian. If you
/// write or read data from/to kernel via OCP, OCP takes care of the
/// right endianness.
#[derive(Debug)]
pub struct OCPRecData {
    msg: Option<String>,
    device_name: Option<String>,
    parent_device_name: Option<String>,
    virt_network_uuid_str: Option<String>,
    // in host endianness!
    node_guid: Option<u64>,
    // in host endianness!
    parent_node_guid: Option<u64>,
}

impl OCPRecData {
    /// Creates a new OCPRecData struct. It parses each attribute that neli received
    /// via generic netlink to its proper Rust runtime type. This is ONLY NECESSARY
    /// for attributes we want to receive.
    pub fn new(res: OveyGeNlMsg) -> Self {
        let mut msg = None;
        let mut device_name = None;
        let mut parent_device_name = None;
        let mut node_guid_be = None;
        let mut parent_node_guid_be = None;
        let mut virt_network_uuid_str = None;

        let payload = res.get_payload().unwrap();

        println!("cmd: {}", payload.cmd);

        payload.get_attr_handle().iter().for_each(|attr| {
            match attr.nla_type {
                OveyAttribute::Msg => {
                    msg.replace(bytes_to_string(attr.nla_payload.as_ref()));
                },
                OveyAttribute::DeviceName => {
                    device_name.replace(bytes_to_string(attr.nla_payload.as_ref()));
                },
                OveyAttribute::ParentDeviceName => {
                    parent_device_name.replace(bytes_to_string(attr.nla_payload.as_ref()));
                },
                OveyAttribute::VirtNetUuidStr => {
                    virt_network_uuid_str.replace(bytes_to_string(attr.nla_payload.as_ref()));
                },
                OveyAttribute::NodeGuid => {
                    node_guid_be.replace(byte_vector_to_u64(attr.nla_payload.as_ref()));
                },
                OveyAttribute::ParentNodeGuid => {
                    parent_node_guid_be.replace(byte_vector_to_u64(attr.nla_payload.as_ref()));
                },
                _ => {}
            }
        });

        OCPRecData {
            msg,
            device_name,
            parent_device_name,
            virt_network_uuid_str,
            // we receive it in big endian format from be;
            // restore host endian format
            node_guid: node_guid_be.map(|u64be| Endianness::u64be_to_u64he(u64be)),
            parent_node_guid: parent_node_guid_be.map(|u64be| Endianness::u64be_to_u64he(u64be)),
        }
    }


    pub fn msg(&self) -> Option<&String> {
        self.msg.as_ref()
    }
    pub fn device_name(&self) -> Option<&String> {
        self.device_name.as_ref()
    }
    pub fn parent_device_name(&self) -> Option<&String> {
        self.parent_device_name.as_ref()
    }
    pub fn virt_network_uuid_str(&self) -> Option<&String> {
        self.virt_network_uuid_str.as_ref()
    }
    pub fn node_guid(&self) -> Option<u64> {
        self.node_guid
    }
    pub fn parent_node_guid(&self) -> Option<u64> {
        self.parent_node_guid
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
        }}",
               self.msg,
               self.device_name,
               self.parent_device_name,
               self.node_guid,
               self.node_guid.map(|val| guid_u64_to_string(val)),
               self.parent_node_guid,
               self.parent_node_guid.map(|val| guid_u64_to_string(val)),
               self.virt_network_uuid_str
        )
    }
}

fn byte_vector_to_u64(bytes: &[u8]) -> u64 {
    assert_eq!(8, bytes.len());

    // let u64_val = payload.as_slice().read_u64::<std::io::>().unwrap();
    // simple Vec<u8> to u64 doesn't work because Rust want to ensure the length
    // of the bytes Array.
    let bytes = [
        bytes[0],
        bytes[1],
        bytes[2],
        bytes[3],
        bytes[4],
        bytes[5],
        bytes[6],
        bytes[7],
    ];
    let u64_val = u64::from_ne_bytes(bytes);

    u64_val
}

/// Useful to turn the bytes from OCP/neli into a real Rust String.
fn bytes_to_string(bytes: &[u8]) -> String {
    let str = String::from_utf8(Vec::from(bytes)).unwrap();
    // Rust doesn't return the null byte by itself
    // it's not a big problem.. but confusing when a Rust
    // String is null terminated.
    let str = String::from(
        str.trim_matches('\0')
    );
    str
}