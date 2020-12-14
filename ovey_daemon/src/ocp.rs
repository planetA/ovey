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
            if res.is_none() {
                continue;
            }
            let res = res.unwrap();

            match res {
                Ok(payload) => {
                    ocp.ocp_resolve_completion(cid);
                }
                Err(err) => {
                    error!("neli reported error while receiving kernel request (netlink/OCP): {}", err.to_string());
                }
            }
        }
    })
}