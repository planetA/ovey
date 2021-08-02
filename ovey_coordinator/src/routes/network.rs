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
    web::Query(query): web::Query<ResolveGidQuery>,
    _req: HttpRequest) -> Result<actix_web::HttpResponse, CoordinatorRestError>
{
    state.with_network(network_uuid, |network| {
        let search_pattern = GidEntry::new(0, query.subnet_prefix, query.interface_id);
        println!("{:#?}", search_pattern);
        println!("{:#?}", network.devices);
        let gid = network.devices.iter()
            .map(|device| device.ports.iter())
            .flatten()
            .map(|port| port.iter_gid())
            .flatten()
            .find(|gid| gid.virt.is_same_addr(&search_pattern))
            .ok_or(CoordinatorRestError::GidNotFound(query.subnet_prefix, query.interface_id))?;

        let output = ResolveGidResp{
            subnet_prefix: gid.real.subnet_prefix,
            interface_id: gid.real.interface_id,
        };
        debug!("Resolve gid: {}: {:#?} {:#?} -> {:#?}", network_uuid, _req,
               query, output);
        Ok(HttpResponse::Ok().json(output))
    })
}
