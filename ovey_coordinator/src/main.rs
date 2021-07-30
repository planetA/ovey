//! Don't get confused. A Rust crate can be/export a library and a binary at the same time.
//! This works because lib.rs and main.rs are handled as separate targets.

#[macro_use]
extern crate log; // import macros

use actix_web::{
    middleware, App, HttpServer,
};
use ovey_coordinator::OVEY_COORDINATOR_PORT;
use config::CONFIG;
use crate::routes::*;

mod config;
mod rest;
mod routes;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Ovey coordinator started with the following initial configuration:");
    println!("{:#?}", *CONFIG);

    std::env::set_var("RUST_LOG", "actix_web=info,debug");
    env_logger::init();

    info!("Starting REST service on localhost:{}", OVEY_COORDINATOR_PORT);

    // println!("Starting REST service on localhost:{}", OVEY_COORDINATOR_PORT);
    let state = new_app_state();

    HttpServer::new(move || {
        App::new()
            // enable logger
            .wrap(middleware::Logger::default())
            .app_data(state.clone())
            .configure(config)
    })
    // TODO also bind the local address? Because this must be called from local network or even remotely?!
        .bind(format!("0.0.0.0:{}", OVEY_COORDINATOR_PORT))?
        .run()
        .await
}
