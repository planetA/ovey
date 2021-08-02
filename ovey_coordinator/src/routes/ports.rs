use actix_web::{HttpResponse, HttpRequest, web};
use rand::prelude::*;
use uuid::Uuid;
use std::convert::TryFrom;

use liboveyutil::urls::ROUTE_PORTS_DEVICE;
use liboveyutil::types::*;

use crate::rest::errors::CoordinatorRestError;
use crate::routes::types::*;

pub(crate) fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource(ROUTE_PORTS_DEVICE)
            .route(web::post().to(route_port_post))
    );
}

/// The coordinator assign new translation address
async fn route_port_post(
    state: web::Data<CoordState>,
    web::Path((network_uuid, device_uuid)): web::Path<(Uuid, Uuid)>,
    web::Query(query): web::Query<CreatePortQuery>,
    _req: HttpRequest) -> Result<actix_web::HttpResponse, CoordinatorRestError>
{
    state.with_network(network_uuid, |network| {
        debug!("Create gd: {}: {:#?} {:#?}", network_uuid, _req, query);

        let device = network.devices.by_device(device_uuid)
            .ok_or(CoordinatorRestError::DeviceUuidNotFound(network_uuid, device_uuid))?;
        // Find the next available index
        // We count port IDs from 1
        let virt_id = u16::try_from(device.ports.len() + 1).unwrap();
        let port = PortEntry::new(Virt::new(query.port, virt_id))
            .set_pkey_tbl_len(query.pkey_tbl_len)
            .set_gid_tbl_len(query.gid_tbl_len)
            .set_core_cap_flags(query.core_cap_flags)
            .set_max_mad_size(query.max_mad_size)
            .to_owned();
        device.ports.push(port);

        let output = CreatePortResp{
            port: virt_id,
	          pkey_tbl_len: query.pkey_tbl_len,
	          gid_tbl_len: query.gid_tbl_len,
	          core_cap_flags: query.core_cap_flags,
	          max_mad_size: query.max_mad_size,
        };
        Ok(HttpResponse::Created().json(output))
    })
}
