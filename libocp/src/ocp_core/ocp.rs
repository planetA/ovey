use neli::socket::NlSocketHandle;
use neli::nl::{Nlmsghdr, NlPayload};
use neli::genl::{Genlmsghdr, Nlattr};
use crate::ocp_properties::{OveyOperation, OveyAttribute, FAMILY_NAME, OcpSocketKind};
use neli::consts::socket::NlFamily;
use neli::utils::U32Bitmask;
use neli::types::{Buffer, GenlBuffer};
use crate::ocp_core::recv::OCPRecData;
use neli::Nl;
use std::fmt::Debug;
use neli::consts::nl::{NlmFFlags, NlmF, Nlmsg};
use liboveyutil::endianness::Endianness;
use neli::err::NlError;
use crate::ocp_core::orchestrator::OcpMessageOrchestrator;
use crate::krequests::KRequest;

pub type OveyNlMsgType = u16;
/// Returned type from neli library when we receive ovey/ocp messages.
pub type OveyGenNetlMsgType = Nlmsghdr<OveyNlMsgType, Genlmsghdr<OveyOperation, OveyAttribute>>;

/// Adapter between our userland code and the Linux kernel module via (generic) netlink.
/// Own abstraction over neli library. It handles the two-socket based async OCP communication
/// between userland (daemon) and the kernel module.
/// **It uses fine-grained internal locking. DON'T EVEN THINK ABOUT ADDING A GLOBAL LOCK AROUND
/// THIS INSTANCE, because it would prevent the async two-socket async communication which is
/// necessary to unblock certain threads.**
pub struct Ocp {
    family_id: u16,
    orchestrator: OcpMessageOrchestrator,
}

impl Ocp {
    /// Starts the connection with the netlink family corresponding to the Ovey Linux kernel module.
    /// * `verbosity` amount of additional output
    pub fn connect() -> Result<Self, String> {
        let mut daemon_to_kernel_socket = NlSocketHandle::connect(
            NlFamily::Generic,
            // we bind the socket (destination) to the kernel (pid 0)
            Some(0),
            U32Bitmask::empty(),
        ).map_err(|e| format!("Socket(daemon_to_kernel_socket): netlink connect failed! err={}", e))?;

        let kernel_to_daemon_socket = NlSocketHandle::connect(
            NlFamily::Generic,
            // we bind the socket (destination) to the kernel (pid 0)
            Some(0),
            U32Bitmask::empty(),
        ).map_err(|e| format!("Socket(kernel_to_daemon_socket): netlink connect failed! err={}", e))?;


        // we could use both sockets here, not important
        let family_id = daemon_to_kernel_socket.resolve_genl_family(FAMILY_NAME).expect("Generic Family must exist!");

        debug!("family id is: {}", family_id);

        // create orchestrator
        let orchestrator = OcpMessageOrchestrator::new(daemon_to_kernel_socket, kernel_to_daemon_socket)?;

        Ok(
            Self {
                family_id,
                orchestrator,
            }
        )
    }

    /// Can be used to receive the next kernel request in an unblocking way. Usually the Daemon
    /// will create a worker thread where this gets invoked in a loop.
    pub fn recv_next_kernel_req_nbl(&self) -> Option<Result<KRequest, NlError>> {
        self.orchestrator.receive_request_from_kernel_nbl()
    }

    /// Usually the Kernel->Daemon socket should be seen as:
    /// take requests from kernel and send a reply. But there
    /// is ONE exception. During the hello/init/startup we need
    /// to tell the kernel what socket is the Kernel->Daemon socket.
    /// Therefore we use in the other direction in this case in a blocking way.
    fn k_to_d_sock_send_req_n_recv_reply_bl(&self,
                                            op: OveyOperation,
                                            attrs: Vec<Nlattr<OveyAttribute, Buffer>>,
    ) -> Result<OCPRecData, String> {
        self.sock_send_req_n_recv_reply_bl(op, attrs, OcpSocketKind::KernelInitiatedRequestsSocket)
    }

    /// Convenient method to send a daemon-initiated request to the kernel
    /// and receive an expected reply in a blocking way.
    fn d_to_k_sock_send_req_n_recv_reply_bl(&self,
                                            op: OveyOperation,
                                            attrs: Vec<Nlattr<OveyAttribute, Buffer>>,
    ) -> Result<OCPRecData, String> {
        self.sock_send_req_n_recv_reply_bl(op, attrs, OcpSocketKind::DaemonInitiatedRequestsSocket)
    }


    /// Convenient function that sends a request via the specified socket
    /// and returns the reply in a blocking way.
    fn sock_send_req_n_recv_reply_bl(&self,
                                     op: OveyOperation,
                                     attrs: Vec<Nlattr<OveyAttribute, Buffer>>,
                                     socket: OcpSocketKind,
    ) -> Result<OCPRecData, String> {
        let nl_msh = self.build_gnlmsg(op, attrs, socket);

        let reply = if socket == OcpSocketKind::DaemonInitiatedRequestsSocket {
            self.orchestrator.send_request_to_kernel(nl_msh)
                .map_err(|e| e.to_string())?;
            self.orchestrator.receive_reply_from_kernel()
        } else {
            self.orchestrator.send_reply_to_kernel(nl_msh)
                .map_err(|e| e.to_string())?;
            self.orchestrator.receive_request_from_kernel_bl()
        };

        let reply = reply
            .map_err(|e| format!("Failed to get reply! {}", e.to_string()))?;

        // eprintln!("res.nlmsg_hdr.nl_pid = {}", reply.nl_pid);

        self.validate(op, &reply)?;

        Ok(
            OCPRecData::new(&reply)
        )
    }


    /// Builds a netlink message (for "neli" lib). It's payload is the generic netlink header.
    /// It's payload is the Ovey data.
    fn build_gnlmsg(&self, op: OveyOperation, attrs: Vec<Nlattr<OveyAttribute, Buffer>>, socket: OcpSocketKind) -> OveyGenNetlMsgType {
        debug!("Sending via netlink socket {:?}: operation={}, all attributes:", socket, op);
        for x in &attrs {
            debug!("    - {} with {} bytes", x.nla_type, x.nla_payload.len());
            // println!("    {} with {:#?}", x.nla_type, x.payload);
        }

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
            // NlmF::Request is needed for messages to get delivered to Ovey kernel module
            // not sure if this is because of neli or because of netlink itself
            // therefore also our
            NlmFFlags::new(&[NlmF::Request]),
            None,
            // This is not used for routing or so. This could be zero for Ovey. But for
            // convenience and to better know from what of the two sockets a specific
            // netlink message came from, this is useful (also for logging)
            Some(u32::from(socket)),
            payload,
        )
    }

    /// Does some validation
    fn validate(&self, op: OveyOperation, res: &OveyGenNetlMsgType) -> Result<(), String> {
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

    /// This function is "fire and forget". It just sends a reply. It doesn't check if
    /// the kernel accepted the reply.
    fn reply_to_kernel(&self, op: OveyOperation, attrs: Vec<Nlattr<OveyAttribute, Buffer>>) {
        let nl_msh = self.build_gnlmsg(
            op,
            attrs,
            OcpSocketKind::KernelInitiatedRequestsSocket,
        );
        self.orchestrator.send_reply_to_kernel(nl_msh).unwrap();
    }

    /// Returns the family id retrieved from the Kernel.
    pub fn family_id(&self) -> u16 {
        self.family_id
    }

    /// Convenient wrapper function that creates an
    /// new Ovey device inside the Ovey Kernel Module
    /// via OCP. It returns whether the operation was
    /// successfully or not.
    pub fn ocp_create_device(&self,
                             device_name: &str,
                             parent_device_name: &str,
                             node_guid_he: u64,
                             network_uuid_str: &str,
    ) -> Result<OCPRecData, String> {
        let node_guid_be = Endianness::u64he_to_u64be(node_guid_he);
        self.d_to_k_sock_send_req_n_recv_reply_bl(
            OveyOperation::CreateDevice,
            vec![
                build_nl_attr(OveyAttribute::DeviceName, device_name),
                build_nl_attr(OveyAttribute::ParentDeviceName, parent_device_name),
                build_nl_attr(OveyAttribute::NodeGuid, node_guid_be),
                build_nl_attr(OveyAttribute::VirtNetUuidStr, network_uuid_str),
            ],
        )
    }

    /// Convenient wrapper function that deletes a n
    /// Ovey device inside the Ovey Kernel Module
    /// via OCP. It returns whether the operation was
    /// successfully or not.
    pub fn ocp_delete_device(&self,
                             device_name: &str,
    ) -> Result<OCPRecData, String> {
        self.d_to_k_sock_send_req_n_recv_reply_bl(
            OveyOperation::DeleteDevice,
            vec![
                build_nl_attr(OveyAttribute::DeviceName, device_name)
            ],
        )
    }

    /// Convenient wrapper function that gets info about an
    /// Ovey device inside the Ovey Kernel Module
    /// via OCP. It returns whether the operation was
    /// successfully or not.
    pub fn ocp_get_device_info(&self,
                               device_name: &str,
    ) -> Result<OCPRecData, String> {
        self.d_to_k_sock_send_req_n_recv_reply_bl(
            OveyOperation::DeviceInfo,
            vec![
                build_nl_attr(OveyAttribute::DeviceName, device_name)
            ],
        )
    }

    /// Convenient wrapper function that tests OCP
    /// with the Kernel Module by sending an ECHO
    /// request. Kernel should reply with an
    /// message with the proper content.
    pub fn ocp_echo(&self,
                    echo_msg: &str,
    ) -> Result<OCPRecData, String> {
        self.d_to_k_sock_send_req_n_recv_reply_bl(
            OveyOperation::Echo,
            vec![
                build_nl_attr(OveyAttribute::Msg, echo_msg)
            ]
        )
    }

    /// Convenient wrapper function that triggers a
    /// error response via OCP by the Ovey Kernel Module.
    pub fn ocp_debug_respond_error(&self) -> Result<OCPRecData, String> {
        self.d_to_k_sock_send_req_n_recv_reply_bl(
            OveyOperation::DebugRespondError,
            vec![],
        )
    }

    /// Convenient function to tell the kernel module that the
    /// two OCP sockets are now available.
    /// Usually triggered during application startup.
    /// The data is send via the corresponding socket.
    ///
    /// THIS IS NECESSARY TO SUPPORT KERNEL-INITIATED REQUESTS.
    // TODO return tuple?!
    pub fn ocp_daemon_hello(&self) -> Result<OCPRecData, String> {
        self.d_to_k_sock_send_req_n_recv_reply_bl(
            OveyOperation::DaemonHello,
            vec![
                build_nl_attr(OveyAttribute::SocketKind, OcpSocketKind::DaemonInitiatedRequestsSocket)
            ],
        )?;
        self.k_to_d_sock_send_req_n_recv_reply_bl(
            OveyOperation::DaemonHello,
            vec![
                build_nl_attr(OveyAttribute::SocketKind, OcpSocketKind::KernelInitiatedRequestsSocket)
            ],
        )
    }

    /// Function is used to tell the kernel module that the
    /// specified socket is no longer available
    /// Usually triggered during application shutdown.
    /// The data is send via the corresponding socket.
    pub fn ocp_daemon_bye(&self) -> Result<OCPRecData, String> {
        self.d_to_k_sock_send_req_n_recv_reply_bl(
            OveyOperation::DaemonBye,
            vec![
                build_nl_attr(OveyAttribute::SocketKind, OcpSocketKind::DaemonInitiatedRequestsSocket)
            ],
        )?;
        self.k_to_d_sock_send_req_n_recv_reply_bl(
            OveyOperation::DaemonBye,
            vec![
                build_nl_attr(OveyAttribute::SocketKind, OcpSocketKind::KernelInitiatedRequestsSocket)
            ],
        )
    }

    /// Function is used to tell the kernel module that the
    /// specified socket is no longer available
    /// Usually triggered during application shutdown.
    /// The data is send via the corresponding socket.
    pub fn ocp_debug_initiate_request(&self) -> (Result<OCPRecData, String>, Result<OCPRecData, String>) {
        (
            self.d_to_k_sock_send_req_n_recv_reply_bl(
                OveyOperation::DebugInitiateRequest,
                vec![],
            ),
            self.orchestrator.receive_request_from_kernel_bl()
                .map(|x| OCPRecData::new(&x))
                .map_err(|e| e.to_string())
        )
    }

    /// Convenient wrapper that tells the kernel to resolve a completion.
    /// This can be seen as a debug function. Real functionality will have more
    /// parameters. This function works as "fire and forget".
    pub fn ocp_resolve_completion(&self, completion_id: u64) {
        self.reply_to_kernel(
            OveyOperation::ResolveCompletion,
            vec![
                build_nl_attr(
                    OveyAttribute::CompletionId,
                    completion_id,
                )],
        );
    }

    /// Convenient wrapper function to tell the kernel via OCP to resolve all
    /// completions. This is useful during debugging if something got stuck.
    pub fn ocp_debug_resolve_all_completions(&self) -> Result<OCPRecData, String> {
        // Actually it's not important what socket we use here..
        // self.d_to_k_sock_send_req_n_recv_reply_bl(
        self.k_to_d_sock_send_req_n_recv_reply_bl(
            OveyOperation::DebugResolveAllCompletions,
            vec![],
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
        payload,
    ).unwrap()
}

/// Convenient function to construct a vector of Nlattr structs to send data.
#[allow(dead_code)]
pub fn build_nl_attrs<T: Nl + Debug>(attr_types: Vec<(OveyAttribute, T)>) -> Vec<Nlattr<OveyAttribute, Buffer>> {
    attr_types.into_iter()
        .map(|x| build_nl_attr(x.0, x.1))
        .collect()
}
