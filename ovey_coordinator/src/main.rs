//! Don't get confused. A Rust crate can be/export a library and a binary at the same time.
//! This works because lib.rs and main.rs are handled as separate targets.

#[macro_use]
extern crate log; // import macros

use actix_web::{
    middleware, web, App, HttpServer,
};
use ovey_coordinator::OVEY_COORDINATOR_PORT;
use config::CONFIG;
use crate::urls::{ROUTE_ADD_DEVICE_URL};
use crate::routes::*;
use ovey_coordinator::urls::*;

mod config;
mod rest;
mod routes;
mod urls;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Ovey coordinator started with the following initial configuration:");
    println!("{:#?}", *CONFIG);

    std::env::set_var("RUST_LOG", "actix_web=info,debug");
    env_logger::init();

    info!("Starting REST service on localhost:{}", OVEY_COORDINATOR_PORT);

    // println!("Starting REST service on localhost:{}", OVEY_COORDINATOR_PORT);

    HttpServer::new(|| {
        App::new()
            // enable logger
            .wrap(middleware::Logger::default())
            .service(web::resource(ROUTE_ADD_DEVICE_URL).route(web::post().to(route_add_device)))
            .service(web::resource(ROUTE_LEASE_GID_URL).route(web::post().to(route_lease_gid)))
            .service(web::resource(ROUTE_RESOLVE_GID_URL).route(web::post().to(route_resolve_gid)))
            .service(web::resource(ROUTE_NETWORK_URL).route(web::get().to(route_get_network_info)))
            .service(web::resource(ROUTE_DEVICE_URL)
                .route(web::delete().to(route_delete_device))
                .route(web::get().to(route_get_device_info)))
            .service(web::resource("/").route(web::get().to(route_index)))
    })
        // TODO also bind the local address? Because this must be called from local network or even remotely?!
        .bind(format!("0.0.0.0:{}", OVEY_COORDINATOR_PORT))?
        .run()
        .await
}

#[cfg(test)]
mod tests {
    /*use super::*;
    use actix_web::dev::Service;
    use actix_web::{http, test, web, App};
    use actix_web::http::Error;*/

    /*#[actix_rt::test]
    async fn test_index() -> Result<(), Error> {
        let mut app = test::init_service(
            App::new().service(web::resource("/").route(web::post().to(route_index))),
        ).await;

        let req = test::TestRequest::post()
            .uri("/")
            .set_json(&MyObj {
                name: "my-name".to_owned(),
                number: 43,
                uuid: None
            })
            .to_request();
        let resp = app.call(req).await.unwrap();

        assert_eq!(resp.status(), http::StatusCode::OK);

        let response_body = match resp.response().body().as_ref() {
            Some(actix_web::body::Body::Bytes(bytes)) => bytes,
            _ => panic!("Response error"),
        };

        assert_eq!(response_body, r##"{"name":"my-name","number":43,"uuid":null}"##);

        Ok(())
    }*/
}
