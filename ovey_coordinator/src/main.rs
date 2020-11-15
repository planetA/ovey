use actix_web::{
    middleware, web, App, HttpServer,
};
use ovey_coordinator::OVEY_COORDINATOR_PORT;
use config::CONFIG;
use crate::routes::urls::{ROUTE_POST_ADD_DEVICE_URL, ROUTE_GET_CONFIG_URL};
use crate::routes::handlers;

mod config;
mod rest;
mod data;
mod db;
mod routes;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Init configuration: supported virtual networks of this coordinator:");
    println!("{:#?}", CONFIG.networks());

    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    println!("Starting REST service on localhost:{}", OVEY_COORDINATOR_PORT);

    HttpServer::new(|| {
        App::new()
            // enable logger
            .wrap(middleware::Logger::default())
            .data(web::JsonConfig::default().limit(4096)) // <- limit size of the payload (global configuration)
            .service(web::resource(ROUTE_GET_CONFIG_URL).route(web::get().to(handlers::route_config)))
            //.service(web::resource("/network/{network}").route(web::get().to(route_add_device)))
            .service(web::resource(ROUTE_POST_ADD_DEVICE_URL).route(web::post().to(handlers::route_add_device)))
            .service(web::resource("/").route(web::get().to(handlers::route_index)))
    })
        .bind(format!("localhost:{}", OVEY_COORDINATOR_PORT))?
        .run()
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::dev::Service;
    use actix_web::{http, test, web, App};

    #[actix_rt::test]
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
    }
}