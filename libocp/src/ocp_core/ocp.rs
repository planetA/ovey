use neli::socket::NlSocketHandle;
use neli::nl::{Nlmsghdr, NlPayload};
use neli::genl::{Genlmsghdr, Nlattr};
use crate::ocp_properties::{OveyOperation, OveyAttribute, FAMILY_NAME};
use neli::consts::socket::NlFamily;
use neli::utils::U32Bitmask;
use neli::types::{Buffer, GenlBuffer};
use crate::ocp_core::recv::OCPRecData;
use neli::Nl;
use std::fmt::Debug;
use neli::consts::nl::{NlmFFlags, NlmF, Nlmsg};
use std::process;
use liboveyutil::endianness::Endianness;

pub type OveyNlMsgType = u16;
/// Returned type from neli library when we receive ovey/ocp messages.
pub type OveyGeNlMsg = Nlmsghdr<OveyNlMsgType, Genlmsghdr<OveyOperation, OveyAttribute>>;

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
                println!("    - {} with {} bytes", x.nla_type, x.nla_payload.len());
                // println!("    {} with {:#?}", x.nla_type, x.payload);
            }
        }

        let nl_msh = self.build_gnlmsg(op, attrs);
        self.socket.send(nl_msh)
            .map_err(|x| format!("Send failed: {}", x))?;

        // Err("foobar".to_string())

        // ack.nl_type == consts::Nlmsg::Error && ack.nl_payload.error == 0
        let res: OveyGeNlMsg = self.socket.recv::<OveyNlMsgType, Genlmsghdr::<OveyOperation, OveyAttribute>>()
            .map_err(|err| err.to_string())?
            .ok_or("No reply received".to_string())?;

        debug!("res.nlmsg_hdr.nl_pid = {}", res.nl_pid);

        self.validate(op, &res)?;

        Ok(
            OCPRecData::new(res)
        )
    }

    /// Builds a netlink message (for "neli" lib). It's payload is the generic netlink header.
    /// It's payload is the Ovey data.
    fn build_gnlmsg(&self, op: OveyOperation, attrs: Vec<Nlattr<OveyAttribute, Buffer>>) -> OveyGeNlMsg {
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

    fn validate(&self, op: OveyOperation, res: &OveyGeNlMsg) -> Result<(), String> {
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
