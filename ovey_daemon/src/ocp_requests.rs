//! Module to handle kernel OCP requests.

use std::thread::{JoinHandle, spawn};
use std::sync::{Arc, Mutex};
use log::{debug, error};
use libocp::krequests::KRequest;
use libocp::ocp_core::Ocp;
use std::sync::atomic::{AtomicBool, Ordering};

/// Starts a thread that continously listens for incoming Ovey kernel module OCP requests.
/// __DAEMON_HELLO__ operation must be sent before this starts. If the daemon is shutting
/// down `exit_work_loop` can be used to gracefully shutdown this thread.
pub fn start_ocp_bg_reply_thread(ocp: Arc<Mutex<Ocp>>, exit_work_loop: Arc<AtomicBool>) -> JoinHandle<()> {
    spawn(move || {
        info!("OCP Kernel request listening loop started in a thread");
        loop {
            if exit_work_loop.load(Ordering::Relaxed) {
                info!("Received signal to exit OCP Kernel request listening loop now.");
                break;
            }

            let mut ocp = ocp.lock().unwrap();
            let res = ocp.recv_next_kernel_req_nbl();
            if let Some(res) = res {
                match res {
                    Ok(kreq) => {
                        if kreq == KRequest::ShutdownDaemon {
                            info!("Received {} from Ovey kernel module. Stopping to listen for Kernel requests.", kreq.op());
                            exit_work_loop.store(true,Ordering::Relaxed);
                            break;
                        }
                        debug!("Received request from Kernel of type {} with completion id {}", kreq.op(), kreq.id());
                        ocp.ocp_resolve_completion(kreq.id());
                    }
                    Err(err) => {
                        error!("neli reported error (netlink/OCP): {}", err.to_string());
                    }
                }
            }
        }
        info!("OCP Kernel request listening loop thread done. Consider restarting Ovey daemon.");
    })
}