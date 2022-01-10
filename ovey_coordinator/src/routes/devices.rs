use actix_web::{HttpResponse, HttpRequest, web};
use uuid::Uuid;

use liboveyutil::urls::ROUTE_DEVICES_ALL;
use liboveyutil::types::*;

use crate::rest::errors::CoordinatorRestError;
use crate::routes::types::*;

pub(crate) fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource(ROUTE_DEVICES_ALL)
            .route(web::get().to(route_devices_get))
    );
}

/// The coordinator assign new translation address
async fn route_devices_get(
    state: web::Data<CoordState>,
    web::Path(network_uuid): web::Path<Uuid>,
    _req: HttpRequest) -> Result<actix_web::HttpResponse, CoordinatorRestError>
{
    state.with_network(network_uuid, |network| {
        debug!("Get all devices: {}: {:#?}", network_uuid, _req);

        Ok(HttpResponse::Created().json(&network.devices))
    })
}
