use actix_web::{HttpResponse, HttpRequest, web};
use rand::prelude::*;
use uuid::Uuid;

use liboveyutil::urls::ROUTE_GIDS_PORT;
use liboveyutil::types::*;

use crate::rest::errors::CoordinatorRestError;
use crate::routes::types::*;

pub(crate) fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource(ROUTE_GIDS_PORT)
            .route(web::post().to(route_gid_post))
            .route(web::put().to(route_gid_put))
    );
}

pub(crate) static DEFULT_GID_PREFIX : u64 = 0xfe80000000000000;

/// The coordinator assign new translation address
async fn route_gid_post(
    state: web::Data<CoordState>,
    web::Path((network_uuid, device_uuid, port_id)): web::Path<(Uuid, Uuid, u16)>,
    web::Json(query): web::Json<LeaseGidQuery>,
    _req: HttpRequest) -> Result<actix_web::HttpResponse, CoordinatorRestError>
{
    state.with_network(network_uuid, |network| {
        if query.gid.is_reserved() && query.gid.is_loopback() {
            return Err(CoordinatorRestError::GidReserved);
        }

        let port : &mut PortEntry = network
            .devices.by_device(device_uuid)
            .ok_or(CoordinatorRestError::DeviceUuidNotFound(network_uuid, device_uuid))?
            .get_port_mut(port_id)
            .ok_or(CoordinatorRestError::PortNotFound(device_uuid, port_id))?;
        debug!("Lease gd: {}: {:#?} {:#?}", network_uuid, _req, query);

        let idx = port.iter_gid()
            .map(|e| e.virt.idx)
            .max()
            .and_then(|idx| Some(idx + 1)).or(Some(0)).unwrap();
        let entry = GidEntry{
            idx: idx,
            gid: Gid{
                subnet_prefix: DEFULT_GID_PREFIX,
                interface_id: random(),
            },
        };
        port.add_gid(Virt{
            virt: entry,
            real: GidEntry{
                idx: query.idx,
                gid: Gid{
                    subnet_prefix: query.gid.subnet_prefix,
                    interface_id: query.gid.interface_id,
                }
            }})?;

        assert_eq!(entry.idx, query.idx);

        let output = LeaseGidResp{
            idx: entry.idx,
            gid: entry.gid,
        };
        Ok(HttpResponse::Ok().json(output))
    })
}

/// The coordinator just remembers the translation, if it is already known
async fn route_gid_put(
    state: web::Data<CoordState>,
    web::Path((network_uuid, device_uuid, port_id)): web::Path<(Uuid, Uuid, u16)>,
    web::Json(query): web::Json<SetGidQuery>,
    _req: HttpRequest) -> Result<actix_web::HttpResponse, CoordinatorRestError>
{
    state.with_network(network_uuid, |network| {
        let new_addr = Virt{
            virt: GidEntry{
                idx: query.virt_idx,
                gid: Gid{
                    subnet_prefix: query.virt.subnet_prefix,
                    interface_id: query.virt.interface_id,
                },
            },
            real: GidEntry{
                idx: query.real_idx,
                gid: Gid{
                    subnet_prefix: query.real.subnet_prefix,
                    interface_id: query.real.interface_id,
                },
            }};

        debug!("Setting an address {:#?}", new_addr);

        // Check if the virtual address has been assigned to the device already.
        // If yes, update the real address mapping.
        if let Some(old_addr) = network
            .devices.by_device(device_uuid)
            .ok_or(CoordinatorRestError::DeviceUuidNotFound(network_uuid, device_uuid))?
            .get_port_mut(port_id)
            .ok_or(CoordinatorRestError::PortNotFound(device_uuid, port_id))?
            .iter_gid_mut()
            .find(|e| e.virt == new_addr.virt) {
                debug!("The entry is already known");
                old_addr.real = new_addr.real;
                let output = SetGidResp{
                    real_idx: new_addr.real.idx,
                    real: new_addr.real.gid,
                    virt_idx: new_addr.virt.idx,
                    virt: new_addr.virt.gid,
                };
                return Ok(HttpResponse::Ok().json(output));
            }

        if !network.is_gid_unique(new_addr) {
            debug!("GID conflict:\n\tNetwork: {}\n\tDevice: {}\n\tPort: {}\n\tAddr: {:#?}",
                   network_uuid, device_uuid, port_id, new_addr);
            for dev in network.devices.iter() {
                debug!("Device: {:#?}", dev);
            }
            return Err(CoordinatorRestError::GidConflict)
        }

        // Find the next available index
        let port = network.devices.by_device(device_uuid)
            .ok_or(CoordinatorRestError::DeviceUuidNotFound(network_uuid, device_uuid))?
            .get_port_mut(port_id)
            .ok_or(CoordinatorRestError::PortNotFound(device_uuid, port_id))?;
        debug!("Let gd: {}: {:#?} {:#?}", network_uuid, _req, query);

        port.add_gid(new_addr)?;

        let output = SetGidResp{
            real_idx: query.real_idx,
            real: query.real,
            virt_idx: query.virt_idx,
            virt: query.virt,
        };
        Ok(HttpResponse::Ok().json(output))
    })
}
