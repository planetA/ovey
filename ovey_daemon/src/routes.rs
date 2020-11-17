//! Crate-private handlers for the REST API for Ovey CLI.

use actix_web::{HttpRequest, HttpResponse, web};
use ovey_daemon::cli_rest_api::{CreateDeviceInput, DeleteDeviceInput};
use crate::coordinator_service::{forward_create_device, forward_delete_device};
use ovey_daemon::cli_rest_api::errors::DaemonRestError;

pub async fn route_get_index(_req: HttpRequest) -> HttpResponse {
    //println!("request: {:?}", &req);
    HttpResponse::Ok().json("STATUS: UP") // <- send response
}

pub async fn route_post_create_device(input: web::Json<CreateDeviceInput>) -> Result<actix_web::HttpResponse, DaemonRestError> {
    // TODO
    //  first step: check if coordinator knows about virtual device in network (if it is allowed)
    //  second:     check if the device is already registered in the coordinator
    //  third:

    let resp = forward_create_device(input.into_inner()).await;
    if resp.is_err() {
        eprintln!("A failure occurred: {:#?}", resp.as_ref().unwrap_err());
    }
    resp.map(|dto| HttpResponse::Ok().json(dto))
}

pub async fn route_delete_delete_device(input: web::Json<DeleteDeviceInput>) -> Result<actix_web::HttpResponse, DaemonRestError> {
    let resp = forward_delete_device(input.into_inner()).await;
    if resp.is_err() {
        eprintln!("A failure occurred: {:#?}", resp.as_ref().unwrap_err());
    }
    resp.map(|dto| HttpResponse::Ok().json(dto))
}
