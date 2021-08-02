use std::time::Instant;
use actix_web::{HttpResponse, HttpRequest, web};
use actix_web::http::StatusCode;
use rand::prelude::*;
use uuid::Uuid;

use liboveyutil::urls::*;
use liboveyutil::types::*;

use crate::rest::errors::CoordinatorRestError;
use crate::routes::types::*;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource(ROUTE_GUIDS_DEVICE)
            .route(web::post().to(route_guid_post))
    );
}

async fn route_guid_post(
    state: web::Data<CoordState>,
    web::Path((network_uuid, device_uuid)): web::Path<(Uuid, Uuid)>,
    web::Query(query): web::Query<LeaseDeviceQuery>,
    _req: HttpRequest) -> Result<actix_web::HttpResponse, CoordinatorRestError>
{
    state.with_network_insert(network_uuid, |network| {

        let (status, virt) = if let Some(mut device) = network.devices.by_device(device_uuid) {
            device.lease = Instant::now();
            (StatusCode::OK, device.guid.unwrap().virt)
        } else {
            let device = DeviceEntry::new(device_uuid)
                .set_guid(Virt{
                    real: query.guid,
                    virt: random::<u64>(),
                }).to_owned();
            let virt = device.guid.unwrap().virt;
            network.devices.insert(device);
            (StatusCode::CREATED, virt)
        };

        debug!("Creating device: {}: {:#?} {:#?}", network_uuid, query, _req);

        let output = LeaseDeviceResp{
            guid: virt,
        };
        Ok(HttpResponse::build(status).json(output))
    })
}
