//! This module describes all functions related to our *Ovey Control Protocol* (OCP).
//! OCP includes all data that is transferred via generic netlink between the user component and
//! the linux kernel module.
//!
//! All functions fall under the constraints of the lib/crate "neli".
//!
//! OCP takes care of guid endianness divergence between kernel and userland.
//! Inside the kernel module the guid is stored as big endian. If you
//! write or read data from/to kernel via OCP, OCP takes care of the
//! right endianness.

use neli::socket::{NlSocket, NlSocketHandle};
use neli::Nl;
use neli::genl::{Genlmsghdr, Nlattr};
use neli::nl::{Nlmsghdr, NlPayload};
use std::{process, fmt};
use std::fmt::{Debug, Display, Formatter};
use serde::{Serialize, Deserialize};

use super::ocp_properties::*;
use liboveyutil::guid::guid_u64_to_string;
use liboveyutil::endianness::Endianness;
use neli::consts::socket::NlFamily;
use neli::utils::U32Bitmask;
use neli::attr::{AttrHandle, Attribute};
use neli::types::{GenlBuffer, Buffer};
use neli::consts::genl::Index;
use neli::consts::nl::{NlmF, NlmFFlags, Nlmsg};
use std::thread::sleep;
use std::time::Duration;

/// Returned type from neli library when we receive ovey/ocp messages.
pub type OveyNlResponse = Nlmsghdr<OveyNlMsgType, Genlmsghdr<OveyOperation, OveyAttribute>>;

/// Struct that holds all the data that can be received via OCP from the kernel. It's up
/// to the caller function to extract the right data.
/// We derive Serialize and Deserialize because it's useful to pass this via REST
/// for debugging. Also, additional compile time overhead is negligible.
///
/// OCP takes care of guid endianness divergence between kernel and userland.
/// Inside the kernel module the guid is stored as big endian. If you
/// write or read data from/to kernel via OCP, OCP takes care of the
/// right endianness.
#[derive(Debug, Serialize, Deserialize)]
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
    pub fn new(res: OveyNlResponse) -> Self {
        let mut msg = None;
        let mut device_name = None;
        let mut parent_device_name = None;
        let mut node_guid_be = None;
        let mut parent_node_guid_be = None;
        let mut virt_network_uuid_str = None;

        let payload = res.get_payload().unwrap();

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

/// The type of a ovey message inside generic netlink. Convenient, simple type instead of
/// the big generic construct.
type OveyGNlMsg = Nlmsghdr<u16, Genlmsghdr<OveyOperation, OveyAttribute>>;

/// Adapter between our userland code and the Linux kernel module via (generic) netlink.
pub struct Ocp {
    family_id: u16,
    socket: NlSocketHandle,
    verbosity: u8,
}

impl Ocp {

    /// Starts the connection with the netlink family corresponding to the Ovey Linux kernel module.
    /// * `verbosity` amount of additional output
    pub fn connect(verbosity: u8) -> Result<Self, String> {
        let mut socket = NlSocketHandle::connect(
            NlFamily::Generic,
             // not 100% sure, probably // 0 => pid of kernel -> socket to kernel?!
            Some(0), // we bind the socket (destination) to the kernel (pid 0)
            U32Bitmask::empty()
        ).map_err(|e| format!("netlink connect failed! err={}", e))?;
        let family_id = socket.resolve_genl_family(FAMILY_NAME).expect("Generic Family must exist!");

        eprintln!("family id is: {}", family_id);

        Ok(
            Self {
                family_id,
                socket,
                verbosity,
            }
        )
    }

    /// Sends a single attribute to kernel and receive the data.
    fn send_single_and_ack<T: Nl + Debug>(&mut self,
                                              op: OveyOperation,
                                              attr_type: OveyAttribute, payload: T) -> Result<OCPRecData, String> {
        let attrs = vec![
            build_nl_attr(attr_type, payload)
        ];
        self.send_and_ack(op, attrs)
    }

    /// Sends a message to the kernel with a vector of data attributes. Ensures that the kernel
    /// replied with an ACK and not an invalid message.
    fn send_and_ack(&mut self,
                        op: OveyOperation,
                        attrs: Vec<Nlattr<OveyAttribute, Buffer>>) -> Result<OCPRecData, String> {
        if self.verbosity > 0 {
            println!("Sending via netlink: operation={}, all attributes:", op);
            for x in &attrs {
                println!("    - {} with {} bytes", x.nla_type, x.nla_payload.size());
                // println!("    {} with {:#?}", x.nla_type, x.payload);
            }
        }

        let nl_msh = self.build_gnlmsg(op, attrs);
        self.socket.send(nl_msh)
            .map_err(|x| format!("Send failed: {}", x))?;

        // Err("foobar".to_string())

        // ack.nl_type == consts::Nlmsg::Error && ack.nl_payload.error == 0
        let res: OveyNlResponse = self.socket.recv::<OveyNlMsgType, Genlmsghdr::<OveyOperation, OveyAttribute>>()
            .map_err(|err| err.to_string())?
            .ok_or("No reply received".to_string())?;

        debug!("res.nlmsg_hdr.nl_pid = {}", res.nl_pid);

        // Do some validation that is useful I think
        // I personally think that recv_ack() by neli is not so good for our
        // purposes;
        self.validate(op, &res)?;

        Ok(
            OCPRecData::new(res)
        )
    }

    /// Builds a netlink message (for "neli" lib). It's payload is the generic netlink header.
    /// It's payload is the Ovey data.
    fn build_gnlmsg(&self, op: OveyOperation, attrs: Vec<Nlattr<OveyAttribute, Buffer>>) -> OveyGNlMsg {
        let mut attrs_buf: GenlBuffer<OveyAttribute, Buffer> = GenlBuffer::new();
        attrs.into_iter().for_each(|a| attrs_buf.push(a));

        // Generic netlink message
        let gen_nl_mhr = Genlmsghdr::new(
            op,
            1, // not important, we don't check this in kernel
            attrs_buf,
        );

        let payload = NlPayload::Payload(gen_nl_mhr);

        // Netlink message
        Nlmsghdr::new(
            None,
            self.family_id,
            // I don't check for flags in the kernel
            NlmFFlags::new(&[NlmF::Request]),
            None,
            // Some(0), // the receiving pid of the packet, 0 is kernel
            Some(process::id()),
            payload
        )
    }

    fn validate(&self, op: OveyOperation, res: &OveyGNlMsg) -> Result<(), String> {
        // res.nl_type is either family id (good message) or NLMSG_ERROR (0x2) for a error message!
        if res.nl_type == u16::from(Nlmsg::Error) /*0x2, same constant is used in the kernel in standard netlink */ {
            return Err("Received Error! Netlink Message Type is \"error\" (2) instead of our family".to_string());
        }

        // should actually never happen, but catch anyway just to be safe
        if res.nl_type != self.family_id {
            return Err(
                format!("Received data from wrong family?! is={}, expected={}", res.nl_type.to_string(), self.family_id)
            );
        };

        if res.nl_payload.get_payload().unwrap().cmd != op {
            return Err(
                format!("Received data (Ack) has wrong operation! is={}, expected={}", res.nl_type.to_string(), self.family_id)
            );
        }

        Ok(())
    }

    /// Returns the family id retrieved from the Kernel.
    pub fn family_id(&self) -> u16 {
        self.family_id
    }

    /// Convenient wrapper function that creates an
    /// new Ovey device inside the Ovey Kernel Module
    /// via OCP. It returns whether the operation was
    /// successfully or not.
    pub fn ocp_create_device(&mut self,
                             device_name: &str,
                             parent_device_name: &str,
                             node_guid_he: u64,
                             network_uuid_str: &str,
    ) -> Result<OCPRecData, String> {
        let node_guid_be = Endianness::u64he_to_u64be(node_guid_he);
        self.send_and_ack(
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
    pub fn ocp_delete_device(&mut self,
                             device_name: &str
    ) -> Result<OCPRecData, String> {
        self.send_and_ack(
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
    pub fn ocp_get_device_info(&mut self,
                               device_name: &str
    ) -> Result<OCPRecData, String> {
        self.send_and_ack(
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
    pub fn ocp_echo(&mut self,
                    echo_msg: &str
    ) -> Result<OCPRecData, String> {
        self.send_single_and_ack(
            OveyOperation::Echo,
            OveyAttribute::Msg,
            echo_msg
        )
    }

    /// Convenient wrapper function that triggers a
    /// error response via OCP by the Ovey Kernel Module.
    pub fn ocp_debug_respond_error(&mut self) -> Result<OCPRecData, String> {
        self.send_and_ack(
            OveyOperation::DebugRespondError,
            vec![]
        )
    }

    /// Convenient wrapper function that triggers a
    /// error response via OCP by the Ovey Kernel Module.
    pub fn ocp_daemon_hello(&mut self) -> Result<OCPRecData, String> {
        self.send_and_ack(
            OveyOperation::DaemonHello,
            vec![]
        )
    }

    /// Convenient wrapper function that triggers a
    /// error response via OCP by the Ovey Kernel Module.
    pub fn ocp_daemon_bye(&mut self) -> Result<OCPRecData, String> {
        self.send_and_ack(
            OveyOperation::DaemonBye,
            vec![]
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

/// Convenient function to construct a Nlattr struct to send data.
pub fn build_nl_attr<T: Nl + Debug>(attr_type: OveyAttribute, payload: T) -> Nlattr<OveyAttribute, Buffer> {
    Nlattr::new(
        // nla_len is updated anyway internally based on payload size
        None,
        false,
        false, // ???
        attr_type,
        payload
    ).unwrap()
}

/// Convenient function to construct a vector of Nlattr structs to send data.
pub fn build_nl_attrs<T: Nl + Debug>(attr_types: Vec<(OveyAttribute, T)>) -> Vec<Nlattr<OveyAttribute, Buffer>> {
    attr_types.into_iter()
        .map(|x| build_nl_attr(x.0, x.1))
        .collect()
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