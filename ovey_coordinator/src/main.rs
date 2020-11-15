use actix_web::{
    middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer,
};
use ovey_coordinator::OVEY_COORDINATOR_PORT;
use config::CONFIG;
use db::DB;
use crate::db::{get_all_data, get_device};
use crate::rest::VirtualizedDeviceInput;

mod config;
mod rest;
mod data;
mod db;

/// This handler uses json extractor
// ", req: HttpRequest" is optional; can be dynamically added; framework takes care of that
/*async fn index(item: web::Json<MyObj>, req: HttpRequest) -> HttpResponse {
    println!("model: {:?}", &item);
    println!("request: {:?}", &req);
    HttpResponse::Ok().json(item.0) // <- send response
}*/

async fn route_config() -> HttpResponse {
    HttpResponse::Ok().json(&*CONFIG) // <- send response
}

async fn route_index(_req: HttpRequest) -> HttpResponse {
    //println!("request: {:?}", &req);
    HttpResponse::Ok().json(get_all_data()) // <- send response
}

async fn route_add_device(input: web::Json<VirtualizedDeviceInput>,
                          web::Path((network_uuid)): web::Path<(uuid::Uuid)>,
                          req: HttpRequest) -> HttpResponse {
    let guid = input.virtual_device_guid_string().to_owned();
    db::add_device_to_network(&network_uuid, input.into_inner());
    let dto = get_device(&network_uuid, &guid).unwrap();
    HttpResponse::Ok().json(dto) // <- send response
}

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
            .service(web::resource("/config").route(web::get().to(route_config)))
            //.service(web::resource("/network/{network}").route(web::get().to(route_add_device)))
            .service(web::resource("/network/{network}/device").route(web::post().to(route_add_device)))
            .service(web::resource("/").route(web::get().to(route_index)))
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