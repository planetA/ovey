use actix_web::{HttpResponse, HttpRequest, web};
use rand::prelude::*;
use uuid::Uuid;

use liboveyutil::urls::{ROUTE_PORTS_DEVICE, ROUTE_PORTS_ONE};
use liboveyutil::types::*;

use crate::rest::errors::CoordinatorRestError;
use crate::routes::types::*;

pub(crate) fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource(ROUTE_PORTS_DEVICE)
            .route(web::post().to(route_port_post))
    );
    cfg.service(
        web::resource(ROUTE_PORTS_ONE)
            .route(web::post().to(route_port_attr_post))
    );
}

/// The coordinator assign new translation address
async fn route_port_post(
    state: web::Data<CoordState>,
    web::Path((network_uuid, device_uuid)): web::Path<(Uuid, Uuid)>,
    web::Json(query): web::Json<CreatePortQuery>,
    _req: HttpRequest) -> Result<actix_web::HttpResponse, CoordinatorRestError>
{
    state.with_network(network_uuid, |network| {
        debug!("Create gd: {}: {:#?} {:#?}", network_uuid, _req, query);

        let device = network.devices.by_device(device_uuid)
            .ok_or(CoordinatorRestError::DeviceUuidNotFound(network_uuid, device_uuid))?;
        let port = device.add_port(query.port)
            .set_pkey_tbl_len(query.pkey_tbl_len)
            .set_gid_tbl_len(query.gid_tbl_len)
            .set_core_cap_flags(query.core_cap_flags)
            .set_max_mad_size(query.max_mad_size);

        let output = CreatePortResp{
            port: port.id.virt,
	          pkey_tbl_len: query.pkey_tbl_len,
	          gid_tbl_len: 2,
	          core_cap_flags: query.core_cap_flags,
	          max_mad_size: query.max_mad_size,
        };
        Ok(HttpResponse::Created().json(output))
    })
}

/// The coordinator assign new translation address
async fn route_port_attr_post(
    state: web::Data<CoordState>,
    web::Path((network_uuid, device_uuid, port_id)): web::Path<(Uuid, Uuid, u16)>,
    web::Json(query): web::Json<SetPortAttrQuery>,
    _req: HttpRequest) -> Result<actix_web::HttpResponse, CoordinatorRestError>
{
    state.with_network(network_uuid, |network| {
        debug!("Set port attributes: {}: {:#?} {:#?}", network_uuid, _req, query);

        let port = network.devices.by_device(device_uuid)
            .ok_or(CoordinatorRestError::DeviceUuidNotFound(network_uuid, device_uuid))?
            .get_port_mut(port_id)
            .ok_or(CoordinatorRestError::PortNotFound(device_uuid, port_id))?;
        // Find the next available index
        // We count port IDs from 1

        let lid = port.lid
            .and_then(|lid| {
                if lid.real == query.lid {
                    Some(lid.virt)
                } else {
                    None
                }
            })
            .or_else(|| {
                port.lid = Some(Virt::new(query.lid, random::<u16>().into()));
                Some(port.lid.unwrap().virt)
            }).unwrap();

        let output = SetPortAttrResp{
            lid: lid,
        };
        debug!("Port attributes: {:#?}", output);
        Ok(HttpResponse::Ok().json(output))
    })
}
