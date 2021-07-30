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
    let mut networks = state.networks.lock().unwrap();
    let network = networks.get_mut(&network_uuid).unwrap();

    let search_pattern = GidEntry::new(0, query.subnet_prefix, query.interface_id);
    println!("{:#?}", search_pattern);
    println!("{:#?}", network.devices);
    let gid = network.devices.iter()
        .filter_map(|device| {
            device.gid.iter().find(|e| e.virt.is_same_addr(&search_pattern))
        })
        .collect::<Vec<_>>();
    if gid.len() == 0 {
        return Ok(HttpResponse::NotFound().finish());
    }

    let output = ResolveGidResp{
        subnet_prefix: gid[0].real.subnet_prefix,
        interface_id: gid[0].real.interface_id,
    };
    debug!("Resolve gid: {}: {:#?} {:#?} -> {:#?}", network_uuid, _req,
           query, output);
    Ok(HttpResponse::Ok().json(output))
}
