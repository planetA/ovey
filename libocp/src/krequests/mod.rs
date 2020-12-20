//! Module for simpler abstractions over incoming kernel requests.

use crate::ocp_core::{OveyGenNetlMsgType};
use crate::ocp_properties::{OveyOperation, OveyAttribute};
use neli::Nl;
use neli::genl::Genlmsghdr;

/// Each completion gets a unique, growing(auto incrementing)
/// completion id assigned.
pub type CompletionId = u64;
/// The request ID is same as [`CompletionId`].
pub type RequestId = CompletionId;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum KRequest {
    /// The kernel sends this request if it wants the daemon to resolve a specific completion
    /// inside the kernel. This is mainly interesting during development since there is no
    /// real useful payload attached to this.
    ResolveCompletion{id: CompletionId},
    /// The kernel sends this to show the daemon the module is unloading. Especially helpful
    /// during development.
    ShutdownDaemon
}

impl From<&OveyGenNetlMsgType> for KRequest {
    fn from(req: &OveyGenNetlMsgType) -> Self {
        let payload = req.get_payload().unwrap();
        let cmd = payload.cmd;
        match cmd {
            OveyOperation::ResolveCompletion => {
                KRequest::ResolveCompletion{
                    id: get_completion_id(payload),
                }
            },
            OveyOperation::KernelModuleBye => {
                KRequest::ShutdownDaemon
            },
            _ => { panic!("Kernel send unknown request: {}", cmd) }
        }
    }
}

impl KRequest {
    /// Convenient getter for the completion id aka request id.
    pub fn id(&self) -> CompletionId {
        match self {
            KRequest::ResolveCompletion{
                id, ..
            } => *id,
            KRequest::ShutdownDaemon => panic!("no ID"),
        }
    }

    /// Convenient getter for the Ovey Operation that created/represents this Kernel requests.
    pub fn op(&self) -> OveyOperation {
        match self {
            KRequest::ResolveCompletion{..} => OveyOperation::ResolveCompletion,
            KRequest::ShutdownDaemon => OveyOperation::KernelModuleBye,
        }
    }
}

fn get_completion_id(payload: &Genlmsghdr<OveyOperation, OveyAttribute>) -> CompletionId {
    let h = payload.get_attr_handle();
    let id = h.get_attribute(OveyAttribute::CompletionId)
        .expect("Kernel Request MUST have a completion id Ovey attribute.");
    let id = u64::deserialize(id.nla_payload.as_ref()).unwrap();
    id
}
