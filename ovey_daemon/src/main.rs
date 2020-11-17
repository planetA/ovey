use ovey_coordinator::rest::structs::*;
use config::CONFIG;
use ovey_coordinator::OVEY_COORDINATOR_PORT;
use ovey_daemon::cli_rest_api::{OVEY_DAEMON_PORT, ROUTE_DEVICE};
use actix_web::{middleware, web, HttpServer, App};
use routes::{route_get_index, route_post_create_device, route_delete_delete_device};
use actix_web::dev::Service;

mod config;
mod routes;
mod coordinator_service;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Ovey daemon started with the following initial configuration:");
    println!("{:#?}", *CONFIG);

    std::env::set_var("RUST_LOG", "actix_web=debug");
    env_logger::init();

    // check if all coordinators are up and valid
    check_config_and_environment().await
        .map_err(|e| {
            eprintln!("{}", e);
            std::io::ErrorKind::Other
        })?;

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

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn builder_works() {
        // see https://crates.io/crates/derive_builder
        let foo = VirtualizedCreateDeviceInputBuilder::default()
            .virtual_device_guid_string("1000:0000:0000:0000")
            .physical_device_guid_string("3000:0000:0000:0000")
            .build()
            .unwrap();
        println!("{:#?}", foo);
    }

}