//! Demo to imitate the daemon.

use libocp::ocp_core::{Ocp, OCPRecData, OcpError};
use std::thread::spawn;
use std::sync::Arc;
use log::{error, debug, info};
use simple_on_shutdown::on_shutdown;
use std::sync::atomic::{AtomicBool, Ordering};
use libocp::krequests::KRequest;

lazy_static::lazy_static! {
    pub(crate) static ref OCP: Arc<Ocp> = {
        Arc::from(
            Ocp::connect().expect("OCP connection must work in order for Ovey daemon to work.")
        )
    };
}

/// Demo to imitate the daemon.
fn main() {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();
    debug!("daemon-like OCP test binary started!");

    let exit_work_loop = Arc::new(AtomicBool::new(false));
    let exit_work_loop_h = exit_work_loop.clone();
    ctrlc::set_handler(move || {
        println!("Received CTRL+C/SIGINT/SIGTERM/SIGKILL");
        exit_work_loop_h.store(true, Ordering::Relaxed);
    }).unwrap();

    // ###########################################
    {
        // init lazy static var
        OCP.ocp_daemon_hello().unwrap();
    }
    on_shutdown!({
        // this will fail, if the request loop received a "kernel module bye"
        // because the netlink destination/socket is gone
        println!("Gracefully shutting down");
        let bye: Result<OCPRecData, OcpError> = OCP.ocp_daemon_bye();
        match bye {
            Ok(_) => { debug!("Daemon sent DaemonBye via OCP") },
            Err(err) => { debug!("DaemonBye via OCP FAILED: probably the kernel module was unloaded (err='{}')", err)  },
        }
    });
    // ###########################################

    let ocp_t = OCP.clone();
    let h = spawn(move || {
        loop {
            if exit_work_loop.load(Ordering::Relaxed) {
                println!("Exiting work loop; this is like when actix event loop in daemon exits");
                break;
            }
            let res = ocp_t.recv_next_kernel_req_nbl();
            if let Some(res) = res {
                match res {
                    Ok(kreq) => {
                        if kreq == KRequest::ShutdownDaemon {
                            info!("OCP told that Ovey Kernel Module is gone. Stopped listening for Kernel OCP requests in daemon.");
                            break;
                        }
                        debug!("Received request from Kernel of type {} with completion id {}", kreq.op(), kreq.id());
                        ocp_t.ocp_resolve_completion(kreq.id());
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
