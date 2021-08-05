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
    web::Json(query): web::Json<ResolveQpGidQuery>,
    _req: HttpRequest) -> Result<actix_web::HttpResponse, CoordinatorRestError>
{
    state.with_network(network_uuid, |network| {
        println!("{:#?}", network.devices);
        println!("Query: {:#?}", query);
        let device = network.devices.iter()
            .find(|device| {
                device.iter_port()
                    .map(|port| port.iter_gid())
                    .flatten()
                    .find(|gid_entry| gid_entry.virt.gid == query.gid)
                    .is_some()
            })
            .ok_or(CoordinatorRestError::GidNotFound(query.gid))?;
        let gid_entry = device.iter_port()
            .map(|port| port.iter_gid())
            .flatten()
            .find(|gid| gid.virt.gid == query.gid)
            .ok_or(CoordinatorRestError::GidNotFound(query.gid))?;
        let qp = device.iter_qps()
            .find(|qp| qp.qpn.virt == query.qpn)
            .ok_or(CoordinatorRestError::QpNotFound(query.qpn))?;

        let output = ResolveQpGidResp{
            gid: gid_entry.real.gid,
            qpn: qp.qpn.real,
        };
        debug!("Resolve gid: {}: {:#?} {:#?} -> {:#?}", network_uuid, _req,
               query, output);
        Ok(HttpResponse::Ok().json(output))
    })
}
