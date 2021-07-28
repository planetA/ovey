//! Module to handle kernel OCP requests.

use std::thread::{JoinHandle, spawn};
use std::sync::{Arc};
use log::{debug, error};
use libocp::krequests::KRequest;
use libocp::ocp_core::Ocp;
use std::sync::atomic::{AtomicBool, Ordering};
use ovey_coordinator::OVEY_COORDINATOR_PORT;
use ovey_coordinator::rest::structs::{VirtualizedDeviceDTO};

// fn store_virt_port_lid(id: u64, real_lid: u32, virt_lid: u32) {
//     debug!("Received a request {} for vLID translation {} -> {}", id, real_lid, virt_lid);

//     // so far: only simulate REST request to measure overhead
//     let res = reqwest::blocking::get(&format!("http://localhost:{}", OVEY_COORDINATOR_PORT));
//     debug!("Received request from the coordinator {:#?}", res);
//     if let Ok(resp) = res {
//         let json = resp.json::<Vec<VirtualizedDeviceDTO>>();
//         if let Ok(json) = json {
//             debug!("Got dummy response from coordinator: {:#?}", json);
//         } else {
//             error!("Dummy response from coordinator failed")
//         }
//     } else {
//         error!("Dummy response from coordinator failed")
//     }

//     debug!("Received request from Kernel with completion id {}", id);
// }

/// Starts a thread that continously listens for incoming Ovey kernel module OCP requests.
/// __DAEMON_HELLO__ operation must be sent before this starts. If the daemon is shutting
/// down `exit_work_loop` can be used to gracefully shutdown this thread.
pub fn start_ocp_bg_reply_thread(ocp: Arc<Ocp>, exit_work_loop: Arc<AtomicBool>) -> JoinHandle<()> {
    spawn(move || {
        info!("OCP Kernel request listening loop started in a thread");
        loop {
            if exit_work_loop.load(Ordering::Relaxed) {
                info!("Received signal to exit OCP Kernel request listening loop now.");
                let bye = ocp.ocp_daemon_bye();
                match bye {
                    Ok(_) => { debug!("Daemon sent DaemonBye via OCP") },
                    Err(err) => {
                        debug!("DaemonBye via OCP FAILED: probably the kernel module was unloaded (err='{}')", err)
                    },
                }
                break;
            }

            let res = ocp.recv_next_kernel_req_nbl();
            match res {
                Some(Ok(kreq)) => {
                    match kreq {
                        KRequest::ResolveCompletion { .. } => {
                            debug!("Received request from Kernel of type {} with completion id {}", kreq.op(), kreq.id());
                            ocp.ocp_resolve_completion(kreq.id());
                        }
                        KRequest::StoreVirtPortLid { id, real_lid, virt_lid } => {
                            panic!("Unimplemented");
                            // store_virt_port_lid(id, real_lid, virt_lid);
                            ocp.ocp_resolve_completion(id);
                        }
                        KRequest::ShutdownDaemon => {
                            info!("Received {} from Ovey kernel module. Stopping to listen for Kernel requests.", kreq.op());
                            exit_work_loop.store(true,Ordering::Relaxed);
                            break;
                        }
                    }

                }
                Some(Err(err)) => {
                    error!("neli reported error (netlink/OCP): {}", err.to_string());
                }
                None => {
                    // Do nothing
                }
            }
        }
        info!("OCP Kernel request listening loop thread done. Consider restarting Ovey daemon.");
    })
}
