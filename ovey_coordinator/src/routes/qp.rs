use actix_web::{HttpResponse, HttpRequest, web};
use rand::prelude::*;
use uuid::Uuid;

use liboveyutil::urls::ROUTE_QPS_DEVICE;
use liboveyutil::types::*;

use crate::rest::errors::CoordinatorRestError;
use crate::routes::types::*;

pub(crate) fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource(ROUTE_QPS_DEVICE)
            .route(web::post().to(route_create_qp))
    );
}

/// The coordinator assign new translation address
async fn route_create_qp(
    state: web::Data<CoordState>,
    web::Path((network_uuid, device_uuid)): web::Path<(Uuid, Uuid)>,
    web::Json(query): web::Json<CreateQpQuery>,
    _req: HttpRequest) -> Result<actix_web::HttpResponse, CoordinatorRestError>
{
    state.with_network(network_uuid, |network| {
        debug!("Create gd: {}: {:#?} {:#?}", network_uuid, _req, query);

        let device = network.devices.by_device(device_uuid)
            .ok_or(CoordinatorRestError::DeviceUuidNotFound(network_uuid, device_uuid))?;
        let virt_qpn = (random::<u32>() + 32) % (1 << 24);
        let _port = device.add_qp(Virt::new(query.qpn, virt_qpn));

        let output = CreateQpResp{
            qpn: virt_qpn,
        };
        Ok(HttpResponse::Created().json(output))
    })
}
