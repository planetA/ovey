//! This module describes all functions related to our *Ovey Control Protocol* (OCP).
//! OCP includes all data that is transferred via generic netlink between the user component and
//! the linux kernel module.
//!
//! All functions fall under the constraints of the lib/crate "neli".

use neli::socket::NlSocket;
use neli::consts::{NlFamily, NlmF};
use neli::Nl;
use neli::genl::Genlmsghdr;
use neli::nl::Nlmsghdr;
use std::{process, fmt};
use std::fmt::{Debug, Display, Formatter};
use neli::nlattr::{Nlattr, AttrHandle};
use liboveyutil::guid::{guid_he_to_string};

use super::ocp_properties::*;

/// Struct that holds all the data that can be received via OCP from the kernel. It's up
/// to the caller function to extract the right data.
pub struct OCPRecData {
    msg: Option<String>,
    device_name: Option<String>,
    parent_device_name: Option<String>,
    virt_network_uuid_str: Option<String>,
    node_guid_be: Option<u64>,
    parent_node_guid_be: Option<u64>,
}

impl OCPRecData {
    /// Creates a new OCPRecData struct. It parses each attribute that neli received
    /// via generic netlink to its proper Rust runtime type. This is ONLY NECESSARY
    /// for attributes we want to receive.
    pub fn new(h: AttrHandle<OveyAttribute>) -> Self {
        let mut msg = None;
        let mut device_name = None;
        let mut parent_device_name = None;
        let mut node_guid_be = None;
        let mut parent_node_guid_be = None;
        let mut virt_network_uuid_str = None;

        h.iter().for_each(|x| {
            let payload = x.payload.clone();
            match x.nla_type {
                OveyAttribute::Msg => {
                    msg.replace(bytes_to_string(payload));
                },
                OveyAttribute::DeviceName => {
                    device_name.replace(bytes_to_string(payload));
                },
                OveyAttribute::ParentDeviceName => {
                    parent_device_name.replace(bytes_to_string(payload));
                },
                OveyAttribute::VirtNetUuidStr => {
                    virt_network_uuid_str.replace(bytes_to_string(payload));
                },
                OveyAttribute::NodeGuid => {
                    node_guid_be.replace(byte_vector_to_u64(payload));
                },
                OveyAttribute::ParentNodeGuid => {
                    parent_node_guid_be.replace(byte_vector_to_u64(payload));
                },
                _ => {}
            }
        });

        OCPRecData {
            msg,
            device_name,
            parent_device_name,
            virt_network_uuid_str,
            node_guid_be,
            parent_node_guid_be,
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
    pub fn guid_be(&self) -> Option<u64> {
        self.node_guid_be
    }
}

impl Display for OCPRecData {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "OCPRecData {{\n\
            \x20   msg: {:?}\n\
            \x20   device_name: {:?}\n\
            \x20   parent_device_name: {:?}\n\
            \x20   guid_be: {:?}\n\
            \x20   guid_string: {:?}\n\
            \x20   parent_guid_be: {:?}\n\
            \x20   parent_guid_string: {:?}\n\
            \x20   virt_network_uuid_str: {:?}\n\
        }}",
               self.msg,
               self.device_name,
               self.parent_device_name,
               self.node_guid_be,
               self.node_guid_be.map(|val| guid_he_to_string(val)),
               self.parent_node_guid_be,
               self.parent_node_guid_be.map(|val| guid_he_to_string(val)),
               self.virt_network_uuid_str
        )
    }
}

/// Adapter between our userland code and the Linux kernel module via (generic) netlink.
pub struct Ocp {
    family_id: u16,
    socket: NlSocket,
    verbosity: u8,
}

/// The type of a ovey message inside generic netlink. Convenient, simple type instead of
/// the big generic construct.
type OveyGNlMsg = Nlmsghdr<u16, Genlmsghdr<OveyOperation, OveyAttribute>>;

impl Ocp {

    /// Starts the connection with the netlink family corresponding to the Ovey Linux kernel module.
    pub fn connect(verbosity: u8) -> Result<Self, String> {
        let mut socket = NlSocket::connect(
            NlFamily::Generic,
            None,
            None,
            // we don't check/use seqs because we don't have data transports that are split into multiple packets
            false
        ).map_err(|e| format!("netlink connect failed! err={}", e))?;
        // Please note that neli hangs in an endless loop when the family doesn't exist as of version
        // 0.4.3. Wait until https://github.com/jbaublitz/neli/issues/87 gets resolved!
        // will probably be very soon resolved with 0.4.4!
        let family_id = socket.resolve_genl_family(FAMILY_NAME).expect("Generic Family must exist!");

        Ok(
            Self {
                family_id,
                socket,
                verbosity
            }
        )
    }

    /// Sends a single attribute to kernel and receive the data.
    pub fn send_single_and_ack<T: Nl + Debug>(&mut self,
                                              op: OveyOperation,
                                              attr_type: OveyAttribute, payload: T) -> Result<OCPRecData, String> {
        let attrs = vec![
            build_nl_attr(attr_type, payload)
        ];
        self.send_and_ack(op, attrs)
    }

    /// Sends a message to the kernel with a vector of data attributes. Ensures that the kernel
    /// replied with an ACK and not an invalid message.
    pub fn send_and_ack(&mut self,
                        op: OveyOperation,
                        attrs: Vec<Nlattr<OveyAttribute, Vec<u8>>>) -> Result<OCPRecData, String> {

        if self.verbosity > 0 {
            println!("Sending via netlink: operation={}, all attributes:", op);
            for x in &attrs {
                println!("    - {} with {} bytes", x.nla_type, x.payload.size());
                // println!("    {} with {:#?}", x.nla_type, x.payload);
            }
        }

        let nl_msh = self.build_gnlmsg(op, attrs);
        self.socket.send_nl(nl_msh)
            .map_err(|x| format!("Send failed: {}", x))?;

        // ack.nl_type == consts::Nlmsg::Error && ack.nl_payload.error == 0
        let res = self.socket.recv_nl::<u16, Genlmsghdr::<OveyOperation, OveyAttribute>>(None).unwrap();

        // Do some validation that is useful I think
        // I personally think that recv_ack() by neli is not so good for our
        // purposes;
        self.validate(op, &res)?;

        Ok(
            OCPRecData::new(
                res.nl_payload.get_attr_handle()
            )
        )
    }

    /// Builds a netlink message (for "neli" lib). It's payload is the generic netlink header.
    /// It's payload is the Ovey data.
    fn build_gnlmsg(&self, op: OveyOperation, attrs: Vec<Nlattr<OveyAttribute, Vec<u8>>>) -> OveyGNlMsg {
        // Generic netlink message
        let gen_nl_mhr = Genlmsghdr::new(
            op,
            1,
            attrs
        ).unwrap();

        // Actually this flags are pretty useless because we don't really check them
        // in our Linux kernel module. But yeah, because by convention we do a request
        // and expect an acknowledgment we just set the proper flags :)
        let nl_msh_flags = vec![
            NlmF::Request,
            NlmF::Ack
        ];

        // Netlink message
        Nlmsghdr::new(
            None,
            self.family_id,
            nl_msh_flags,
            None,
            Some(process::id()),
            gen_nl_mhr
        )
    }

    fn validate(&self, op: OveyOperation, res: &OveyGNlMsg) -> Result<(), String> {
        // res.nl_type is either family id (good message) or NLMSG_ERROR (0x2) for a error message!
        if res.nl_type == 2 /*Nlmsg::Error as u16*/ {
            return Err("Received Error! Netlink Message Type is \"error\" (2) instead of our family".to_string());
        }

        // should actually never happen, but catch anyway just to be safe
        if res.nl_type != self.family_id {
            return Err(
                format!("Received data from wrong family?! is={}, expected={}", res.nl_type.to_string(), self.family_id)
            );
        };

        if res.nl_payload.cmd != op {
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
}

fn byte_vector_to_u64(bytes: Vec<u8>) -> u64 {
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
pub fn build_nl_attr<T: Nl + Debug>(attr_type: OveyAttribute, payload: T) -> Nlattr<OveyAttribute, Vec<u8>> {
    Nlattr::new(
        // nla_len is updated anyway internally based on payload size
        None,
        attr_type,
        payload
    ).unwrap()
}

/// Convenient function to construct a vector of Nlattr structs to send data.
pub fn build_nl_attrs<T: Nl + Debug>(attr_types: Vec<(OveyAttribute, T)>) -> Vec<Nlattr<OveyAttribute, Vec<u8>>> {
    attr_types.into_iter()
        .map(|x| build_nl_attr(x.0, x.1))
        .collect()
}

/// Useful to turn the bytes from OCP/neli into a real Rust String.
fn bytes_to_string(bytes: Vec<u8>) -> String {
    let str = String::from_utf8(bytes).unwrap();
    // Rust doesn't return the null byte by itself
    // it's not a big problem.. but confusing when a Rust
    // String is null terminated.
    let str = String::from(
        str.trim_matches('\0')
    );
    str
}