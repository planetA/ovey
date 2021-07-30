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
    let mut networks = state.networks.lock().unwrap();
    let network = networks.get_mut(&network_uuid).unwrap();
    debug!("Lease gd: {}: {:#?} {:#?}", network_uuid, _req, query);

    if let Some(device) = network.devices.by_device(device_uuid) {
        // Find the next available index
        let idx = device.gid.iter()
            .map(|e| e.virt.idx)
            .max()
            .and_then(|idx| Some(idx + 1)).or(Some(0)).unwrap();
        let gid = GidEntry{
            port: 1,
            idx: idx,
            subnet_prefix: random(),
            interface_id: random(),
        };
        device.set_gid(Virt{
            virt: gid,
            real: GidEntry{
                port: query.port,
                idx: query.idx,
                subnet_prefix: query.subnet_prefix,
                interface_id: query.interface_id,
            }});

        assert_eq!(gid.port, query.port);
        assert_eq!(gid.idx, query.idx);

        let output = LeaseGidResp{
            port: gid.port,
            idx: gid.idx,
            subnet_prefix: gid.subnet_prefix,
            interface_id: gid.interface_id,
        };
        Ok(HttpResponse::Ok().json(output))
    } else {
        Ok(HttpResponse::NotFound().finish())
    }
}

/// The coordinator just remembers the translation, if it is already known
async fn route_gid_put(
    state: web::Data<CoordState>,
    web::Path((network_uuid, device_uuid)): web::Path<(Uuid, Uuid)>,
    web::Query(query): web::Query<SetGidQuery>,
    _req: HttpRequest) -> Result<actix_web::HttpResponse, CoordinatorRestError>
{
    let mut networks = state.networks.lock().unwrap();
    let network = networks.get_mut(&network_uuid).unwrap();
    debug!("Let gd: {}: {:#?} {:#?}", network_uuid, _req, query);

    if let Some(device) = network.devices.by_device(device_uuid) {
        // Find the next available index
        // TODO: Need to check that the virtual and real addresses are unique
        device.set_gid(Virt{
            virt: GidEntry{
                port: query.virt_port,
                idx: query.virt_idx,
                subnet_prefix: query.virt_subnet_prefix,
                interface_id: query.virt_interface_id,
            },
            real: GidEntry{
                port: query.real_port,
                idx: query.real_idx,
                subnet_prefix: query.real_subnet_prefix,
                interface_id: query.real_interface_id,
            }});

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
    } else {
        Ok(HttpResponse::NotFound().finish())
    }
}
