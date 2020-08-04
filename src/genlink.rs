//! This module describes our interface to generic netlink. It lets us send and receive
//! data related to our specific domain via netlink. All functions fall under the constraints of
//! the lib/crate "neli".

use neli::socket::NlSocket;
use neli::consts::{NlFamily, NlmF, Cmd, NlAttrType};
use neli::Nl;
use neli::genl::Genlmsghdr;
use neli::nl::Nlmsghdr;
use std::{process, fmt};
use std::fmt::{Debug, Display, Formatter};
use neli::nlattr::Nlattr;

// Implements the necessary trait for the "neli" lib on an enum called "OveyOperation".
// Command corresponds to "enum OveyOperation" in kernel module C code.
// Describes what callback function shall be invoked in the linux kernel module.
impl_var_trait!(
    OveyOperation, u8, Cmd,
    Unspec => 0,
    Echo => 1,
    CreateDevice => 2,
    DeleteDevice => 3
);
impl Display for OveyOperation {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            OveyOperation::Unspec => write!(f, "OveyOperation::Unspec"),
            OveyOperation::Echo => write!(f, "OveyOperation::Echo"),
            OveyOperation::CreateDevice => write!(f, "OveyOperation::CreateDevice"),
            OveyOperation::DeleteDevice => write!(f, "OveyOperation::DeleteDevice"),
            _ =>  write!(f, "OveyOperation::<unknown>"),
        }
    }
}

// Implements the necessary trait for the "neli" lib on an enum called "OveyAttribute".
// Command corresponds to "enum OveyAttribute" in kernel module C code.
// Describes the value type to data mappings inside the generic netlink packet payload.
impl_var_trait!(
    OveyAttribute, u16, NlAttrType,
    Unspec => 0,
    Msg => 1,
    DeviceName => 2,
    ParentDeviceName => 3
);
impl Display for OveyAttribute {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            OveyAttribute::Unspec => write!(f, "OveyAttribute::Unspec"),
            OveyAttribute::Msg => write!(f, "OveyAttribute::Msg"),
            OveyAttribute::DeviceName => write!(f, "OveyAttribute::DeviceName"),
            OveyAttribute::ParentDeviceName => write!(f, "OveyAttribute::ParentDeviceName"),
            _ =>  write!(f, "OveyAttribute::<unknown>"),
        }
    }
}

/// Adapter between our userland code and the Linux kernel module via (generic) netlink.
pub struct GenlinkAdapter {
    family_id: u16,
    socket: NlSocket,
}

impl GenlinkAdapter {

    /// Starts the connection with the netlink family corresponding to the Ovey Linux kernel module.
    pub fn connect(family_name: &str) -> Self {
        let mut socket = NlSocket::connect(
            NlFamily::Generic,
            None,
            None,
            true
        ).expect("netlink connect failed!");
        // Please note that neli hangs in an endless loop when the family doesn't exist as of version
        // 0.4.3. Wait until https://github.com/jbaublitz/neli/issues/87 gets resolved!
        // will probably be very soon resolved with 0.4.4!
        let family_id = socket.resolve_genl_family(family_name).expect("Generic Family must exist!");
        Self {
            family_id,
            socket
        }
    }

    /// Sends a single attribute to kernel.
    pub fn send_single(&mut self, op: OveyOperation, attr: Nlattr<OveyAttribute, Vec<u8>>) {
        let attrs = vec![attr];
        self.send(op, attrs);
    }

    /// Sends a message to the kernel with a vector of data attributes.
    pub fn send(&mut self, op: OveyOperation, attrs: Vec<Nlattr<OveyAttribute, Vec<u8>>>) {
        println!("Sending via netlink: operation={}", op);
        println!("    all attributes:");
        for x in &attrs {
            println!("    - {} with {} bytes", x.nla_type, x.payload.size());
            // println!("    {} with {:#?}", x.nla_type, x.payload);
        }

        let gen_nl_mhr = Genlmsghdr::new(
            op,
            1,
            attrs
        ).unwrap();
        let nl_msh_flags = vec![NlmF::Request];
        let nl_msh = Nlmsghdr::new(
            None,
            self.family_id,
            nl_msh_flags,
            None,
            Some(process::id()),
            gen_nl_mhr
        );
        self.socket.send_nl(nl_msh).unwrap();
    }

    /// Receives all attributes that kernel sent.
    pub fn recv_all(&mut self) -> Vec<Nlattr<OveyAttribute, Vec<u8>>> {
        // u16: family type :)
        let res = self.socket.recv_nl::<u16, Genlmsghdr::<OveyOperation, OveyAttribute>>(None).unwrap();

        if self.family_id != res.nl_type {
            // todo not quite sure yet what's the meaning of the nl_type
            //  I think it always should be family number.. but there were cases
            //  where it was 2 (NETLINK_USERSOCK).
            println!("Received data from wrong family?! is={}, expected={}", res.nl_type, self.family_id);
        };

        let mut data = vec![];
        res.nl_payload.get_attr_handle().iter().for_each(|x|
            data.push(
                Nlattr::new(
                    Some(x.nla_len),
                    x.nla_type.clone(),
                    x.payload.clone())
                .unwrap()
            )
        );
        data
    }

    /// Returns the raw (not deserialized) data from the first attribute of the specified type.
    pub fn recv_first_of_type_raw(&mut self, attr_type: OveyAttribute) -> Option<Vec<u8>> {
        let data = self.recv_all();
        let ele = data.into_iter()
            .filter(|x| x.nla_type == attr_type)
            .last();
        ele.map(|x| x.payload)
    }


    /// Returns the family id retrieved from the Kernel.
    pub fn family_id(&self) -> u16 {
        self.family_id
    }
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
