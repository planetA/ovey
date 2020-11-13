use actix_web::{
    middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
struct MyObj {
    name: String,
    number: i32,
    uuid: Option<Uuid>
}

/// This handler uses json extractor
// ", req: HttpRequest" is optional; can be dynamically added; framework takes care of that
async fn index(item: web::Json<MyObj>, req: HttpRequest) -> HttpResponse {
    println!("model: {:?}", &item);
    println!("request: {:?}", &req);
    HttpResponse::Ok().json(item.0) // <- send response
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    HttpServer::new(|| {
        App::new()
            // enable logger
            .wrap(middleware::Logger::default())
            .data(web::JsonConfig::default().limit(4096)) // <- limit size of the payload (global configuration)
            .service(web::resource("/extractor").route(web::post().to(index)))
            .service(web::resource("/").route(web::post().to(index)))
    })
        .bind("127.0.0.1:8080")?
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
            App::new().service(web::resource("/").route(web::post().to(index))),
        )
            .await;

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