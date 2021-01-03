use std::sync::mpsc::{Receiver, sync_channel, SyncSender, TryRecvError};
use crate::ocp_core::ocp::OveyGenNetlMsgType;
use neli::err::NlError;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use neli::socket::NlSocketHandle;
use std::thread::spawn;
use crate::krequests::KRequest;

/// Orchestrates all messages. OCP messages can either be from Userland(Daemon) to Kernel
/// or from Kernel to Userland(Daemon).
/// **It uses fine-grained internal locking. DON'T EVEN THINK ABOUT ADDING A GLOBAL LOCK AROUND
/// THIS INSTANCE, because it would prevent the async two-socket async communication which is
/// necessary to unblock certain threads.**
pub struct OcpMessageOrchestrator {

    /// Receiver of a channel used to receive Kernel initiated requests from a worker thread.
    kernel_request_channel_receiver: Mutex<Receiver<Result<OveyGenNetlMsgType, NlError>>>,

    /// This boolean is set by the main thread to signal the child thread to gracefully exit
    worker_thread_should_stop: Arc<AtomicBool>,

    /// Used to send daemon-initiated requests and receive kernel replies.
    daemon_to_kernel_socket: Mutex<NlSocketHandle>,

    /// Used to receive kernel-initiated requests and send Ovey userland replies.
    kernel_to_daemon_socket: Arc<Mutex<NlSocketHandle>>,
}

impl OcpMessageOrchestrator {
    pub fn new(daemon_to_kernel_socket: NlSocketHandle,
               kernel_to_daemon_socket: NlSocketHandle,
    ) -> Result<Self, String> {
        let (sender, receiver) = sync_channel(1);

        let x = OcpMessageOrchestrator {
            kernel_request_channel_receiver: Mutex::new(receiver),
            daemon_to_kernel_socket: Mutex::new(daemon_to_kernel_socket),
            // this is an Arc because we need this in a worker thread for the orchestrator
            kernel_to_daemon_socket: Arc::new(Mutex::new(kernel_to_daemon_socket)),
            worker_thread_should_stop: Arc::new(AtomicBool::new(false))
        };
        x.init_kernel_to_daemon_receive_thread(sender);
        Ok(x)
    }

    fn init_kernel_to_daemon_receive_thread(&self, sender: SyncSender<Result<OveyGenNetlMsgType, NlError>>) {
        // because we don't know when to expect requests from kernel on this socket
        // we make it nonblocking. This way we can multiplex it to send or receive
        // messages easily.

        // release lock after this line! Important!
        self.kernel_to_daemon_socket.lock().unwrap().nonblock().unwrap();

        let should_stop = self.worker_thread_should_stop.clone();
        let socket = self.kernel_to_daemon_socket.clone();
        spawn(move || {
            loop {
                let should_stop = should_stop.load(Ordering::Relaxed);
                if should_stop { break }

                // we don't hold the lock permanently because we also want to allow
                // using the socket for sending
                let mut socket = socket.lock().unwrap();
                // this is non blocking because we marked the socket as nonblocking earlier
                let res = socket.recv();

                // now check if we actually received something
                // because this is non blocking there is no guarantee
                // TODO this probably needs refactoring ... lets check what happens here
                //  with nonblocking receive
                if let Err(ref err) = res {
                    error!("Received error from Kernel->Daemon Netlink socket: {}", err);
                    // panic!("Aborting. Because at this point we can't decide if a non blocking ", err);
                }
                let res = res.unwrap();
                if res.is_none() {
                    // this is a valid case. This happens if no result was found
                    // because we received nonblocking
                    //debug!("Received empty response from Kernel->Daemon Netlink socket",);
                }
                else {
                    let res: OveyGenNetlMsgType = res.unwrap();
                    debug!("Successfully received request on Kernel->daemon socket");
                    sender.send(Ok(res)).unwrap();
                }
            }
            debug!("Gracefully stopped worked thread of OcpMessageOrchestrator");
            debug!("Stopped receiving OCP messages by Ovey kernel module");
        });
    }

    /// Sends a single request to the Kernel via OCP.
    /// This function operates on `self.daemon_to_kernel_socket`
    pub fn send_request_to_kernel(&self, msg: OveyGenNetlMsgType) -> Result<(), NlError> {
        let mut socket = self.daemon_to_kernel_socket.lock().unwrap();
        socket.send(msg)
    }

    /// Sends a single reply to the Kernel via OCP.
    /// This function operates on `self.kernel_to_daemon_socket`
    pub fn send_reply_to_kernel(&self, msg: OveyGenNetlMsgType) -> Result<(), NlError> {
        debug!("Replying to kernel");
        let mut socket = self.kernel_to_daemon_socket.lock().unwrap();
        socket.send(msg)
    }

    /// Receives a single reply from the kernel in a blocking way.
    /// Call this if you previously send a request where you
    /// expect an reply.
    /// This function operates on `self.daemon_to_kernel_socket`
    pub fn receive_reply_from_kernel_bl(&self) -> Result<OveyGenNetlMsgType, NlError> {
        let mut socket = self.daemon_to_kernel_socket.lock().unwrap();
        // we unwrap because we wait for packages blocking
        // therefore there is no None() and always Some()
        socket.recv().map(|x| x.unwrap())
    }

    /// Receives a single request from the kernel in a blocking way.
    /// Call this if you want to handle kernel-initiated communication/requests.
    /// This function operates on `self.kernel_to_daemon_socket`
    pub fn receive_request_from_kernel_bl(&self) -> Result<OveyGenNetlMsgType, NlError> {
        // blocking until a value can be received
        let receiver = self.kernel_request_channel_receiver.lock().unwrap();
        receiver.recv().unwrap()
    }

    /// Receives a single request from the kernel in a non-blocking way.
    /// Call this if you want to handle kernel-initiated communication/requests.
    /// This function operates on `self.kernel_to_daemon_socket`
    pub fn receive_request_from_kernel_nbl(&self) -> Option<Result<KRequest, NlError>> {
        // nonblocking
        let receiver = self.kernel_request_channel_receiver.lock().unwrap();
        let res = receiver.try_recv();
        if let Err(err) = res {
            return match err {
                TryRecvError::Empty => {
                    // No message received; okay because we wait for messages non blocking
                    None
                }
                TryRecvError::Disconnected => {
                    error!("Channel is disconnected");
                    // TODO panic?
                    None
                    // No message received; okay because we wait for messages non blocking
                }
            }
        }
        let res= res.unwrap();

        Some(
            res.map(|ref msg| KRequest::from(msg))
        )
    }
}

impl Drop for OcpMessageOrchestrator {
    fn drop(&mut self) {
        self.worker_thread_should_stop.store(true, Ordering::Relaxed);
    }
}
