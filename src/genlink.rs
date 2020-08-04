//! This module describes our interface to generic netlink that provides functions for our
//! own protocol on top of generic netlink.

use neli::socket::NlSocket;
use neli::consts::{NlFamily, NlmF, Cmd, NlAttrType};
use neli::Nl;
use neli::genl::Genlmsghdr;
use neli::nl::Nlmsghdr;
use std::{process, fmt};
use std::fmt::{Debug, Display, Formatter};
use neli::nlattr::Nlattr;

// Implements the necessary trait for the "neli" lib on an enum called "Command".
// Command corresponds to "enum Commands" in kernel module C code.
// Describes what callback function shall be invoked in the linux kernel module.
impl_var_trait!(
    Command, u8, Cmd,
    Unspec => 0,
    Echo => 1
);
impl Display for Command {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Command::Unspec => write!(f, "Command::Unspec"),
            Command::Echo => write!(f, "Command::Echo"),
            _ =>  write!(f, "Command::<unknown>"),
        }
    }
}

// Implements the necessary trait for the "neli" lib on an enum called "ControlAttr".
// Command corresponds to "enum Attributes" in kernel module C code.
// Describes the value type to data mappings inside the generic netlink packet payload.
impl_var_trait!(
    ControlAttr, u16, NlAttrType,
    Unspec => 0,
    Msg => 1
);
impl Display for ControlAttr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ControlAttr::Unspec => write!(f, "ControlAttr::Unspec"),
            ControlAttr::Msg => write!(f, "ControlAttr::Msg"),
            _ =>  write!(f, "ControlAttr::<unknown>"),
        }
    }
}

pub struct GenlinkAdapter {
    family_id: u16,
    socket: NlSocket,
}

impl GenlinkAdapter {
    pub fn connect(family_name: &str) -> Self {
        let mut socket = NlSocket::connect(
            NlFamily::Generic,
            None,
            None,
            true
        ).expect("netlink connect failed!");
        let family_id = socket.resolve_genl_family(family_name).expect("Generic Family must exist!");
        Self {
            family_id,
            socket
        }
    }

    pub fn send<T: Nl + Debug>(&mut self, attr_type: ControlAttr, payload: T) {
        println!("Sending '{}' with Payload {:?} via netlink", attr_type, payload);
        let attr = Nlattr::new(
            Some(payload.size() as u16),
            attr_type,
            payload
        ).unwrap();
        let attrs = vec![attr];
        let gen_nl_mhr = Genlmsghdr::new(Command::Echo, 1, attrs).unwrap();
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

    pub fn recv_all(&mut self) -> Vec<Nlattr<ControlAttr, Vec<u8>>> {
        // u16: family type :)
        let res = self.socket.recv_nl::<u16, Genlmsghdr::<Command, ControlAttr>>(None).unwrap();
        assert_eq!(self.family_id, res.nl_type, "Received data from wrong family?! is={}, expected={}", res.nl_type, self.family_id);

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

    pub fn recv_first_msg(&mut self) -> Option<String> {
        // u16: family type :)
        let res = self.socket.recv_nl::<u16, Genlmsghdr::<Command, ControlAttr>>(None).unwrap();
        assert_eq!(self.family_id, res.nl_type, "Received data from wrong family?! is={}, expected={}", res.nl_type, self.family_id);

        let mut ret: Option<String> = None;
        // iterate through all received attributes
        res.nl_payload.get_attr_handle().iter().for_each(|xattr| {
            if ret.is_none() && ControlAttr::Msg == xattr.nla_type {
                ret.replace(
                    String::from_utf8(xattr.payload.clone()).unwrap()
                );
            }
        });

        ret
    }

    pub fn family_id(&self) -> u16 {
        self.family_id
    }
}
