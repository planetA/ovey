//! Crate-private handlers for the REST API of Ovey Daemon. They get invoked by Ovey CLI.

use crate::coordinator_service::{
    rest_check_device_is_allowed,
};
use crate::OCP;
use actix_web::{web, HttpRequest, HttpResponse};
use libocp::ocp_core::{OCPRecData, OcpError};
use liboveyutil::guid::{guid_string_to_u64, guid_u64_to_string};
use liboveyutil::lid::{lid_string_to_u16, lid_u16_to_string};
use liboveyutil::types::Uuid;
use ovey_daemon::errors::DaemonRestError;
use ovey_daemon::structs::{
    CreateDeviceInput, CreateDeviceInputBuilder, DeleteDeviceInput, DeleteDeviceInputBuilder,
    DeletionStateDto, DeletionStateDtoBuilder, DeviceInfoDto,
};
use std::str::FromStr;
use log::debug;

pub async fn route_get_index(_req: HttpRequest) -> HttpResponse {
    //println!("request: {:?}", &req);
    HttpResponse::Ok().json("STATUS: UP") // <- send response
}

