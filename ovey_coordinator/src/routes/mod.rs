//! Handles all routes/controllers. Functions that get invoked on a specific route.

use actix_web::{HttpResponse, HttpRequest, web};

use crate::rest::structs::VirtualizedDeviceInput;
use crate::db::{get_all_data, add_device_to_network, get_device};
use crate::config::CONFIG;
use crate::rest::errors::CoordinatorRestError;

pub async fn route_config() -> HttpResponse {
    HttpResponse::Ok().json(&*CONFIG) // <- send response
}

pub async fn route_index(_req: HttpRequest) -> HttpResponse {
    //println!("request: {:?}", &req);
    HttpResponse::Ok().json(get_all_data()) // <- send response
}

pub async fn route_add_device(input: web::Json<VirtualizedDeviceInput>,
                              web::Path(network_uuid): web::Path<uuid::Uuid>,
                              _req: HttpRequest) -> Result<actix_web::HttpResponse, CoordinatorRestError> {
    let guid = input.virtual_device_guid_string().to_owned();
    add_device_to_network(&network_uuid, input.into_inner())?;
    let dto = get_device(&network_uuid, &guid).unwrap();
    Ok(HttpResponse::Ok().json(dto))
}
