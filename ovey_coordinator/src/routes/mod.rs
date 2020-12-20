//! Handles all routes/controllers. Functions that get invoked on a specific route from
//! Ovey daemon requests.

use actix_web::{HttpResponse, HttpRequest, web};

use crate::rest::structs::VirtualizedDeviceInput;
use crate::db::{db_get_all_data, db_add_device_to_network, db_get_device_data, db_delete_device_from_network, db_get_network_data};
use crate::config::CONFIG;
use crate::rest::errors::CoordinatorRestError;
use liboveyutil::types::{GuidIdType, Uuid};

pub async fn route_config() -> HttpResponse {
    HttpResponse::Ok().json(&*CONFIG) // <- send response
}

pub async fn route_index(_req: HttpRequest) -> HttpResponse {
    //println!("request: {:?}", &req);
    HttpResponse::Ok().json(db_get_all_data()) // <- send response
}

pub async fn route_get_network_info(web::Path(network_uuid): web::Path<Uuid>)
  -> Result<actix_web::HttpResponse, CoordinatorRestError> {
    db_get_network_data(&network_uuid)
        .map(|vec| HttpResponse::Ok().json(vec))
}

pub async fn route_get_device_info(web::Path((network_uuid, virt_dev_id)): web::Path<(Uuid, GuidIdType)>)
    -> Result<actix_web::HttpResponse, CoordinatorRestError> {
    db_get_device_data(&network_uuid, &virt_dev_id)
        .map(|dto| HttpResponse::Ok().json(dto))
}

pub async fn route_add_device(input: web::Json<VirtualizedDeviceInput>,
                              web::Path(network_uuid): web::Path<Uuid>,
                              _req: HttpRequest) -> Result<actix_web::HttpResponse, CoordinatorRestError> {
    let dto = db_add_device_to_network(&network_uuid, input.into_inner())?;
    Ok(HttpResponse::Ok().json(dto))
}


pub async fn route_delete_device(web::Path((network_uuid, virt_dev_id)): web::Path<(Uuid, GuidIdType)>)
    -> Result<actix_web::HttpResponse, CoordinatorRestError> {
    db_delete_device_from_network(&network_uuid, &virt_dev_id)
        .map(|dto| HttpResponse::Ok().json(dto))
}
