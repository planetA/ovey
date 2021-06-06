//! Handles all routes/controllers. Functions that get invoked on a specific route from
//! Ovey daemon requests.

use actix_web::{HttpResponse, HttpRequest, web};

use crate::config::CONFIG;
use crate::rest::errors::CoordinatorRestError;
use uuid::Uuid;
use liboveyutil::types::*;

pub async fn route_config() -> HttpResponse {
    HttpResponse::Gone().finish() // <- send response
}

pub async fn route_index(_req: HttpRequest) -> HttpResponse {
    //println!("request: {:?}", &req);
    HttpResponse::Gone().finish() // <- send response
}

pub async fn route_get_network_info(web::Path(network_uuid): web::Path<Uuid>)
  -> Result<actix_web::HttpResponse, CoordinatorRestError> {
    Ok(HttpResponse::Gone().finish())
}

pub async fn route_get_device_info(web::Path((network_uuid, virt_dev_id)): web::Path<(Uuid, GuidString)>)
    -> Result<actix_web::HttpResponse, CoordinatorRestError> {
    Ok(HttpResponse::Gone().finish())
}

pub async fn route_add_device(
    input: web::Json<LeaseDeviceReq>,
    web::Path(network_uuid): web::Path<Uuid>,
    _req: HttpRequest) -> Result<actix_web::HttpResponse, CoordinatorRestError>
{
    let input: LeaseDeviceReq = input.into_inner();
    debug!("Creating device: {}: {:#?} {:#?}", network_uuid, _req, input);

    let output = LeaseDeviceResp{
        guid: input.guid,
    };
    Ok(HttpResponse::Ok().json(output))
}

pub async fn route_lease_gid(
    input: web::Json<LeaseGidReq>,
    web::Path(network_uuid): web::Path<Uuid>,
    _req: HttpRequest) -> Result<actix_web::HttpResponse, CoordinatorRestError>
{
    let input: LeaseGidReq = input.into_inner();
    debug!("Creating device: {}: {:#?} {:#?}", network_uuid, _req, input);

    let output = LeaseGidResp{
        port: input.port,
        idx: input.idx,
        subnet_prefix: input.subnet_prefix.into(),
        interface_id: input.interface_id.into(),
    };
    Ok(HttpResponse::Ok().json(output))
}


pub async fn route_delete_device(web::Path((network_uuid, virt_dev_id)): web::Path<(Uuid, GuidString)>)
    -> Result<actix_web::HttpResponse, CoordinatorRestError> {
    Ok(HttpResponse::Gone().finish())
}
