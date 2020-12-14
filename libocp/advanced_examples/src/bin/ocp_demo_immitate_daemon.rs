//! Demo to imitate the daemon.

use libocp::ocp_core::Ocp;
use std::thread::spawn;
use std::sync::{Arc, Mutex};
use log::{error, debug};
use simple_on_shutdown::on_shutdown;
use std::sync::atomic::{AtomicBool, Ordering};

lazy_static::lazy_static! {
    pub(crate) static ref OCP: Arc<Mutex<Ocp>> = {
        Arc::from(
            Mutex::from(
                Ocp::connect().expect("OCP connection must work in order for Ovey daemon to work.")
            )
        )
    };
}

/// Demo to imitate the daemon.
fn main() {
    let exit_work_loop = Arc::new(AtomicBool::new(false));
    let exit_work_loop_h = exit_work_loop.clone();
    ctrlc::set_handler(move || {
        println!("Received CTRL+C/SIGINT/SIGTERM/SIGKILL");
        exit_work_loop_h.store(true, Ordering::Relaxed);
    }).unwrap();

    // ###########################################
    {
        // init lazy static var
        let mut ocp = OCP.lock().unwrap();
        ocp.ocp_daemon_hello().unwrap();
    }
    on_shutdown!({
        println!("Gracefully shutting down");
        OCP.lock().unwrap().ocp_daemon_bye().unwrap();
    });
    // ###########################################

    let ocp_t = OCP.clone();
    let h = spawn(move || {
        loop {
            if exit_work_loop.load(Ordering::Relaxed) {
                println!("Exiting work loop; this is like when actix event loop in daemon exits");
                break;
            }
            let mut ocp = ocp_t.lock().unwrap();
            let res = ocp.recv_next_kernel_req_nbl();
            if let Some(res) = res {
                match res {
                    Ok(kreq) => {
                        debug!("Received request from Kernel of type {} with completion id {}", kreq.op(), kreq.id());
                        ocp.ocp_resolve_completion(kreq.id());
                    }
                    Err(err) => {
                        error!("neli reported error (netlink/OCP): {}", err.to_string());
                    }
                }
            }
        }
    });

    h.join().unwrap();
    println!("Child thread exited");
}
