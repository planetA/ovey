use actix_web::{HttpResponse, HttpRequest, web};
use rand::prelude::*;
use uuid::Uuid;

use liboveyutil::urls::ROUTE_GIDS_DEVICE;
use liboveyutil::types::*;

use crate::rest::errors::CoordinatorRestError;
use crate::routes::types::*;

pub(crate) fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource(ROUTE_GIDS_DEVICE)
            .route(web::post().to(route_gid_post))
            .route(web::put().to(route_gid_put))
    );
}

/// The coordinator assign new translation address
async fn route_gid_post(
    state: web::Data<CoordState>,
    web::Path((network_uuid, device_uuid)): web::Path<(Uuid, Uuid)>,
    web::Query(query): web::Query<LeaseGidQuery>,
    _req: HttpRequest) -> Result<actix_web::HttpResponse, CoordinatorRestError>
{
    state.with_network(network_uuid, |network| {
        let port : &mut PortEntry = network
            .devices.by_device(device_uuid)
            .ok_or(CoordinatorRestError::DeviceUuidNotFound(network_uuid, device_uuid))?
            .ports.iter_mut().find(|p| p.id.real == query.port)
            .ok_or(CoordinatorRestError::PortNotFound(device_uuid, query.port))?;
        debug!("Lease gd: {}: {:#?} {:#?}", network_uuid, _req, query);

        let idx = port.iter_gid()
            .map(|e| e.virt.idx)
            .max()
            .and_then(|idx| Some(idx + 1)).or(Some(0)).unwrap();
        let gid = GidEntry{
            idx: idx,
            subnet_prefix: random(),
            interface_id: random(),
        };
        port.add_gid(Virt{
            virt: gid,
            real: GidEntry{
                idx: query.idx,
                subnet_prefix: query.subnet_prefix,
                interface_id: query.interface_id,
            }});

        assert_eq!(gid.idx, query.idx);

        let output = LeaseGidResp{
            port: port.id.virt,
            idx: gid.idx,
            subnet_prefix: gid.subnet_prefix,
            interface_id: gid.interface_id,
        };
        Ok(HttpResponse::Ok().json(output))
    })
}

/// The coordinator just remembers the translation, if it is already known
async fn route_gid_put(
    state: web::Data<CoordState>,
    web::Path((network_uuid, device_uuid)): web::Path<(Uuid, Uuid)>,
    web::Query(query): web::Query<SetGidQuery>,
    _req: HttpRequest) -> Result<actix_web::HttpResponse, CoordinatorRestError>
{
    state.with_network(network_uuid, |network| {
        let new_addr = Virt{
            virt: GidEntry{
                idx: query.virt_idx,
                subnet_prefix: query.virt_subnet_prefix,
                interface_id: query.virt_interface_id,
            },
            real: GidEntry{
                idx: query.real_idx,
                subnet_prefix: query.real_subnet_prefix,
                interface_id: query.real_interface_id,
            }};

        if !network.is_gid_unique(new_addr) {
            return Err(CoordinatorRestError::GidConflict)
        }

        // Find the next available index
        let port = network.devices.by_device(device_uuid)
            .ok_or(CoordinatorRestError::DeviceUuidNotFound(network_uuid, device_uuid))?
            .ports.iter_mut().find(|p| ((p.id.real == query.real_port) && (p.id.virt == query.virt_port)))
            .ok_or(CoordinatorRestError::PortNotFound(device_uuid, query.real_port))?;
        debug!("Let gd: {}: {:#?} {:#?}", network_uuid, _req, query);

        port.add_gid(new_addr);

        let output = SetGidResp{
            real_port: query.virt_port,
            real_idx: query.virt_idx,
            real_subnet_prefix: query.virt_subnet_prefix,
            real_interface_id: query.virt_interface_id,
            virt_port: query.real_port,
            virt_idx: query.real_idx,
            virt_subnet_prefix: query.real_subnet_prefix,
            virt_interface_id: query.real_interface_id,
        };
        Ok(HttpResponse::Ok().json(output))
    })
}
