//! Crate-private handlers for the REST API for Ovey CLI.

use actix_web::{HttpRequest, HttpResponse};

pub async fn route_get_index(_req: HttpRequest) -> HttpResponse {
    //println!("request: {:?}", &req);
    HttpResponse::Ok().json("Up") // <- send response
}

pub async fn route_post_create_device(_req: HttpRequest) -> HttpResponse {
    //println!("request: {:?}", &req);
    HttpResponse::Ok().json("Up") // <- send response
}

pub async fn route_delete_delete_device(_req: HttpRequest) -> HttpResponse {
    //println!("request: {:?}", &req);
    HttpResponse::Ok().json("Up") // <- send response
}
