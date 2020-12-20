//! Module to handle kernel OCP requests.

use libocp::ocp_core::{Ocp, OCPRecData};
use std::thread::{JoinHandle, spawn};
use std::sync::{Arc, Mutex};
use log::{debug, error};
use libocp::ocp_properties::{OveyOperation, OveyAttribute};
use libocp::krequests::KRequest;

pub fn start_ocp_bg_reply_thread(ocp: Arc<Mutex<Ocp>>) -> JoinHandle<()> {
    spawn(move || {
        loop {
            let mut ocp = ocp.lock().unwrap();
            let res = ocp.recv_next_kernel_req_nbl();
            if let Some(res) = res {
                match res {
                    Ok(kreq) => {
                        if kreq == KRequest::ShutdownDaemon {
                            info!("OCP told that Ovey Kernel Module is gone. Stopped listening for Kernel OCP requests in daemon.");
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
        info!("OCP Kernel request thread done. Consider restarting daemon.");
    })
}