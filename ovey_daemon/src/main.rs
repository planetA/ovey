use config::CONFIG;
use crate::coordinator_service::check_config_and_environment;
use ovey_daemon::urls::{ROUTE_DEVICE, ROUTE_DEVICES};
use std::sync::Arc;
use libocp::ocp_core::{Ocp, OcpError};
use libocp::ocp_core::OCPRecData;
use simple_on_shutdown::on_shutdown_move;
use crate::ocp_krequests::start_ocp_bg_reply_thread;
use std::sync::atomic::{AtomicBool, Ordering};
use futures::executor::block_on;

mod config;
mod coordinator_service;
mod ocp_krequests;

#[macro_use]
extern crate log;

lazy_static::lazy_static! {
    pub(crate) static ref OCP: Arc<Ocp> = {
        Arc::from(
            Ocp::connect().expect("OCP connection must work in order for Ovey daemon to work.")
        )
    };
}

fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    debug!("Ovey daemon started with the following initial configuration:");
    debug!("{:#?}", *CONFIG);

    // init lazy static OCP + tell kernel daemon started
    {
        let _ = OCP.ocp_daemon_hello().expect("should work");
        debug!("Daemon told kernel via OCP hello");
    }
    // We use this var to notify the Kernel request listening loop (OCP)
    let exit_loop = Arc::new(AtomicBool::new(false));
    let loop_thread_handle = start_ocp_bg_reply_thread(OCP.clone(), exit_loop.clone());

    // Important that this value lives through the whole lifetime of main()
    on_shutdown_move!({
        // wait for thread to finish
        exit_loop.store(true, Ordering::Relaxed);
        debug!("thread finished");
    });

    // check if all coordinators are up and valid
    block_on(check_config_and_environment())?;
    loop_thread_handle.join().unwrap();

    Ok(())
}
