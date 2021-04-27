use config::CONFIG;
use actix_web::{middleware, web, HttpServer, App};
use crate::coordinator_service::check_config_and_environment;
use routes::{route_get_index, route_post_create_device, route_delete_delete_device};
use ovey_daemon::consts::OVEY_DAEMON_PORT;
use ovey_daemon::urls::{ROUTE_DEVICE, ROUTE_DEVICES};
use std::sync::Arc;
use libocp::ocp_core::{Ocp, OcpError};
use libocp::ocp_core::OCPRecData;
use simple_on_shutdown::on_shutdown_move;
use crate::ocp_krequests::start_ocp_bg_reply_thread;
use std::sync::atomic::{AtomicBool, Ordering};
use crate::routes::route_get_list_devices;

mod config;
mod coordinator_service;
mod routes;
mod util;
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info,debug");
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
        loop_thread_handle.join().unwrap();
        debug!("thread finished");
        let bye: Result<OCPRecData, OcpError> = OCP.ocp_daemon_bye();
        match bye {
            Ok(_) => { debug!("Daemon sent DaemonBye via OCP") },
            Err(err) => { debug!("DaemonBye via OCP FAILED: probably the kernel module was unloaded (err='{}')", err)  },
        }
    });

    // check if all coordinators are up and valid
    check_config_and_environment().await
        .map_err(|e| {
            eprintln!("{}", e);
            std::io::ErrorKind::Other
        })?;

    info!("Starting REST service on localhost:{}", OVEY_DAEMON_PORT);

    HttpServer::new(|| {
        App::new()
            // enable logger
            .wrap(middleware::Logger::default())
            // use default value .data(web::JsonConfig::default().limit(4096)) // <- limit size of the payload (global configuration)
            .service(
                web::resource(ROUTE_DEVICES)
                    .route(web::get().to(route_get_list_devices))
            )
            .service(
                web::resource(ROUTE_DEVICE)
                    .route(web::post().to(route_post_create_device))
                    .route(web::delete().to(route_delete_delete_device))
            )
            .service(web::resource("/").route(web::get().to(route_get_index)))
    })
        // I think this bind already should prevent public access and only allow localhost?!
        .bind(format!("localhost:{}", OVEY_DAEMON_PORT))?
        .run()
        .await
}
