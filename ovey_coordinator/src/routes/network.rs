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
            .route(web::get().to(route_resolve_gid))
    );
}

async fn route_resolve_gid(
    state: web::Data<CoordState>,
    web::Path(network_uuid): web::Path<Uuid>,
    web::Json(query): web::Json<ResolveQpQuery>,
    _req: HttpRequest) -> Result<actix_web::HttpResponse, CoordinatorRestError>
{
    state.with_network(network_uuid, |network| {
        println!("{:#?}", network.devices);
        println!("Query: {:#?}", query);
        let virt_gid = query.gid
            .ok_or(CoordinatorRestError::GidReserved)?;
        let device = network.devices.iter()
            .find(|device| {
                device.iter_port()
                    .map(|port| port.iter_gid())
                    .flatten()
                    .find(|gid_entry| gid_entry.virt.gid == virt_gid)
                    .is_some()
            })
            .ok_or(CoordinatorRestError::GidNotFound(virt_gid))?;
        let gid_entry = if let Some(virt_gid) = query.gid {
            Some(device.iter_port()
                 .map(|port| port.iter_gid())
                 .flatten()
                 .find(|gid| gid.virt.gid == virt_gid)
                 .ok_or(CoordinatorRestError::GidNotFound(virt_gid))?)
        } else {
            None
        };
        let qp = if let Some(virt_qpn) = query.qpn {
            Some(device.iter_qps()
                .find(|qp| qp.qpn.virt == virt_qpn)
                .ok_or(CoordinatorRestError::QpNotFound(virt_qpn))?)
        } else {
            None
        };
        let lid_entry = if let Some(virt_lid) = query.lid {
            Some(device.iter_port()
                 .filter_map(|port| port.lid)
                 .find(|lid| lid.virt == virt_lid)
                 .ok_or(CoordinatorRestError::LidNotFound(virt_lid))?)
        } else {
            None
        };

        let output = ResolveQpResp{
            gid: gid_entry.and_then(|g| Some(g.real.gid)),
            qpn: qp.and_then(|q| Some (q.qpn.real)),
            lid: lid_entry.and_then(|l| Some(l.real)),
        };
        debug!("Resolve gid: {}: {:#?} {:#?} -> {:#?}", network_uuid, _req,
               query, output);
        Ok(HttpResponse::Ok().json(output))
    })
}
