use ovey_coordinator::rest::structs::*;
use config::CONFIG;
use actix_web::{middleware, web, HttpServer, App};
use routes::{route_get_index, route_post_create_device, route_delete_delete_device};
use ovey_coordinator::OVEY_COORDINATOR_PORT;
use ovey_daemon::consts::OVEY_DAEMON_PORT;
use ovey_daemon::urls::ROUTE_DEVICE;
use std::sync::Mutex;
use libocp::ocp_core::Ocp;

mod config;
mod coordinator_service;
mod routes;
mod util;

lazy_static::lazy_static! {
    pub(crate) static ref OCP: Mutex<Ocp> = {
        Mutex::from(Ocp::connect(4, true).expect("OCP connection must work in order for Ovey daemon to work."))
    };
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Ovey daemon started with the following initial configuration:");
    println!("{:#?}", *CONFIG);

    std::env::set_var("RUST_LOG", "actix_web=info,debug");
    env_logger::init();

    // check if all coordinators are up and valid
    check_config_and_environment().await
        .map_err(|e| {
            eprintln!("{}", e);
            std::io::ErrorKind::Other
        })?;

    // init lazy static OCP
    {
        let _ = OCP.lock().unwrap();
    }

    println!("Starting REST service on localhost:{}", OVEY_DAEMON_PORT);

    HttpServer::new(|| {
        App::new()
            // enable logger
            .wrap(middleware::Logger::default())
            // use default value .data(web::JsonConfig::default().limit(4096)) // <- limit size of the payload (global configuration)
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

async fn check_config_and_environment() -> Result<(), String> {
    let mut actual_up = 0;
    let expected_up = CONFIG.coordinators().len();

    if expected_up == 0 {
        return Err("There is not a single Ovey coordinator configured.".to_owned());
    }

    // check all hosts are available
    for (network, host) in CONFIG.coordinators() {
        let mut host = host.to_owned();
        // e.g. http://localhost:13337
        host.push_str(&format!(":{}", OVEY_COORDINATOR_PORT));

        let resp = reqwest::get(&host).await;
        let resp = resp.map_err(|_| format!("Ovey coordinator on configured host '{}' IS NOT UP.", &host))?;
        let resp = resp.json::<AllNetworksDtoType>().await;
        let json = resp.map_err(|_| format!("Ovey coordinator @ host '{}' sent invalid response.", &host))?;

        if json.contains_key(network) {
            actual_up += 1;
        } else {
            eprintln!(
                "Ovey coordinator on configured host '{}' does not know about network '{}'!",
                &host,
                network
            );
        }
    }

    return if actual_up != expected_up {
        Err("WARNING: Not all Ovey coordinators are available.".to_owned())
    } else {
        println!("INFO: All Ovey coordinators are available.");
        Ok(())
    }
}
