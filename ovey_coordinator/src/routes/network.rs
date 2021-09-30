/// Network-level endpoints

use actix_web::{HttpResponse, HttpRequest, web};
use uuid::Uuid;

use liboveyutil::urls::{ROUTE_GIDS_ALL};
use liboveyutil::types::*;

use crate::rest::errors::CoordinatorRestError;
use crate::routes::types::*;

pub(crate) fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource(ROUTE_GIDS_ALL)
            .route(web::get().to(route_resolve_qp))
    );
}

fn resolve_gid(network: &NetworkState, query_gid: Option<Gid>) -> Result<Option<(&DeviceEntry, Virt<GidEntry>)>, CoordinatorRestError>
{
    if query_gid.is_none() {
        return Ok(None);
    }
    let query_gid = query_gid.unwrap();
    let device = network.devices.iter()
        .find(|device| {
            device.iter_port()
                .map(|port| port.iter_gid())
                .flatten()
                .find(|gid_entry| gid_entry.virt.gid == query_gid)
                .is_some()
        })
        .ok_or(CoordinatorRestError::GidNotFound(query_gid))?;
    Ok(Some((device, *device.iter_port()
         .map(|port| port.iter_gid())
         .flatten()
         .find(|gid| gid.virt.gid == query_gid)
         .ok_or(CoordinatorRestError::GidNotFound(query_gid))?)))
}

fn resolve_lid(network: &NetworkState, query_lid: Option<u32>) -> Result<Option<(&DeviceEntry, Virt<u32>)>, CoordinatorRestError>
{
    if query_lid.is_none() {
        return Ok(None);
    }
    let query_lid = query_lid.unwrap();

    let device = network.devices.iter()
        .find(|device| {
            device.iter_port()
                .find(|port| port.lid.is_some() && port.lid.unwrap().virt == query_lid)
                .is_some()
        })
        .ok_or(CoordinatorRestError::LidNotFound(query_lid))?;
    Ok(Some((device, device.iter_port()
             .find_map(|port| if port.lid.is_some() && port.lid.unwrap().virt == query_lid {port.lid} else {None})
             .ok_or(CoordinatorRestError::LidNotFound(query_lid))?)))
}

fn resolve_qpn(device: &DeviceEntry, query_qpn: Option<u32>) -> Result<Option<&QpEntry>, CoordinatorRestError> {
    if query_qpn.is_none() {
        return Ok(None);
    }

    Ok(Some(device.iter_qps()
         .find(|qp| qp.qpn.virt == query_qpn.unwrap())
         .ok_or(CoordinatorRestError::QpNotFound(query_qpn.unwrap()))?))
}

async fn route_resolve_qp(
    state: web::Data<CoordState>,
    web::Path(network_uuid): web::Path<Uuid>,
    web::Json(query): web::Json<ResolveQpQuery>,
    _req: HttpRequest) -> Result<actix_web::HttpResponse, CoordinatorRestError>
{
    state.with_network(network_uuid, |network| {
        println!("{:#?}", network.devices);
        println!("Query: {:#?}", query);
        let gid_search = resolve_gid(network, query.gid)?;
        let lid_search = resolve_lid(network, query.lid)?;

        let qp_search = match (gid_search, lid_search) {
            (Some((device, _)), None) => resolve_qpn(device, query.qpn)?,
            (None, Some((device, _))) => resolve_qpn(device, query.qpn)?,
            (Some((device_a, _)), Some((device_b, _))) => {
                if device_a.device != device_b.device {
                    return Err(CoordinatorRestError::DeviceConflict);
                }
                resolve_qpn(device_a, query.qpn)?
            },
            (None, None) => if let Some(qpn) = query.qpn {
                return Err(CoordinatorRestError::QpNotFound(qpn));
            } else {
                None
            },
        };

        let output = ResolveQpResp{
            gid: gid_search.and_then(|g| Some(g.1.real.gid)),
            lid: lid_search.and_then(|l| Some(l.1.real)),
            qpn: qp_search.and_then(|q| Some (q.qpn.real)),
        };
        debug!("Resolve gid: {}: {:#?} {:#?} -> {:#?}", network_uuid, _req,
               query, output);
        Ok(HttpResponse::Ok().json(output))
    })
}
