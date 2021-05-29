//! Crate-private handlers for the REST API of Ovey Daemon. They get invoked by Ovey CLI.

use crate::coordinator_service::{
    rest_check_device_is_allowed, rest_forward_delete_device,
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
use ovey_daemon::util::get_all_local_ovey_devices;
use std::str::FromStr;
use log::debug;

pub async fn route_get_index(_req: HttpRequest) -> HttpResponse {
    //println!("request: {:?}", &req);
    HttpResponse::Ok().json("STATUS: UP") // <- send response
}

pub async fn route_get_list_devices() -> Result<actix_web::HttpResponse, DaemonRestError> {
    let devs = get_all_local_ovey_devices();
    debug!("Available local ovey devices: {:#?}", &devs);

    // the ? operator inside map seems not to work :/
    let devs = devs
        .into_iter()
        .map(|dev| {
            OCP.ocp_get_device_info(&dev)
                .map_err(|e| DaemonRestError::OcpDeviceNotFound {
                    info: format!(
                        "could not fetch info for device '{}' via OCP. err='{}'",
                        dev, e
                    ),
                })
        })
        .collect::<Vec<Result<OCPRecData, DaemonRestError>>>();

    // check if there is any error and unwrap the first one
    let errs = devs
        .iter()
        .filter(|x| x.is_err())
        .map(|x| x.as_ref().unwrap_err())
        .collect::<Vec<&DaemonRestError>>();
    if !errs.is_empty() {
        // return error
        return Err(errs[0].clone());
    }

    // now that we know there are no errors unwrap all real objects
    let devs = devs
        .into_iter()
        .map(|x| x.unwrap())
        .collect::<Vec<OCPRecData>>();

    let devs = devs
        .into_iter()
        .map(|data| {
            DeviceInfoDto{
                dev_name: data.device_name().unwrap().to_string(),
                parent_dev_name: data.parent_device_name().unwrap().to_string(),
                guid: data.node_guid().unwrap(),
                lid: data.node_lid().unwrap(),
                parent_guid: data.parent_node_guid().unwrap(),
                virtual_network_id: Uuid::from_str(data.virt_network_uuid_str().unwrap()).unwrap()
            }
        })
        .collect::<Vec<DeviceInfoDto>>();

    Ok(HttpResponse::Ok().json(devs))
}
