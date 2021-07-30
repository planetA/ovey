//! Handles all routes/controllers. Functions that get invoked on a specific route from
//! Ovey daemon requests.

use actix_web::{HttpResponse, HttpRequest, web};

use crate::rest::errors::CoordinatorRestError;
use uuid::Uuid;
use std::collections::HashMap;
use std::sync::Mutex;
use rand::prelude::*;
use liboveyutil::urls::*;
use liboveyutil::types::*;

mod types;
mod guids;

use types::*;

pub(crate) fn new_app_state() -> web::Data<CoordState> {
    web::Data::new(CoordState{
        networks: Mutex::new(HashMap::new())
    })
}

pub fn config(cfg: &mut web::ServiceConfig) {
    guids::config(cfg);
    cfg.service(
        web::resource(ROUTE_GIDS_ALL)
            .route(web::get().to(route_resolve_gid))
    ).service(
        web::resource(ROUTE_GIDS_DEVICE)
            .route(web::post().to(route_gid_post))
            .route(web::put().to(route_gid_put))
    );
}

pub(crate) async fn route_gid_post(
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

pub(crate) async fn route_resolve_gid(
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

pub(crate) async fn route_gid_put(
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

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App};
    use actix_web::test::TestRequest;
    use actix_web::http::StatusCode;

    const GUID: u64 = 444;

    #[actix_rt::test]
    async fn build_lease_device_request() {
        let state = new_app_state();
        let mut app = test::init_service(
            App::new()
                .app_data(state.clone())
                .configure(config)).await;
        let network = Uuid::new_v4();

        let device = Uuid::new_v4();
        let query = LeaseDeviceQuery{
            guid: GUID,
        };
        let uri = query.compile(None, network, Some(device));
        let req = TestRequest::with_uri(&uri)
            .method(query.method())
            .to_request();
        println!("{:#?}", req);
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), StatusCode::CREATED);
        let body = test::read_body(resp).await;
        let resp1_struct: LeaseDeviceResp = serde_json::from_slice(&body).unwrap();
        assert_ne!(GUID, resp1_struct.guid);

        let req = TestRequest::with_uri(&uri)
            .method(query.method())
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = test::read_body(resp).await;
        let resp2_struct: LeaseDeviceResp = serde_json::from_slice(&body).unwrap();
        println!("{:#?}", resp2_struct);
        assert_eq!(resp2_struct.guid, resp1_struct.guid);
    }

    #[actix_rt::test]
    async fn build_new_gid_request() {
        let state = new_app_state();
        let mut app = test::init_service(
            App::new()
                .app_data(state.clone())
                .configure(config)).await;
        let network_uuid = Uuid::new_v4();
        let device_uuid = Uuid::new_v4();
        let real_subnet_prefix: u64 = 4;
        let real_interface_id: u64 = 5;

        let query = LeaseDeviceQuery{
            guid: GUID,
        };
        let uri = query.compile(None, network_uuid, Some(device_uuid));
        println!("{}", uri);
        let req = TestRequest::with_uri(&uri)
            .method(query.method())
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), StatusCode::CREATED);
        let body = test::read_body(resp).await;
        let guid_struct: LeaseDeviceResp = serde_json::from_slice(&body).unwrap();
        assert_ne!(GUID, guid_struct.guid);

        let query = LeaseGidQuery{
            port: 1,
            idx: 0,
            subnet_prefix: real_subnet_prefix,
            interface_id: real_interface_id,
        };
        let uri = query.compile(None, network_uuid, Some(device_uuid));
        let req = TestRequest::with_uri(&uri)
            .method(query.method())
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = test::read_body(resp).await;
        let gid_struct: LeaseGidResp = serde_json::from_slice(&body).unwrap();
        println!("{:#?}", gid_struct);
        assert_eq!(gid_struct.port, query.port);
        assert_eq!(gid_struct.idx, query.idx);
        assert_ne!(gid_struct.subnet_prefix, query.subnet_prefix);
        assert_ne!(gid_struct.interface_id, query.interface_id);

        let query = ResolveGidQuery{
            subnet_prefix: gid_struct.subnet_prefix,
            interface_id: gid_struct.interface_id,
        };
        let uri = query.compile(None, network_uuid, None);
        println!("{}", uri);
        let req = TestRequest::with_uri(&uri)
            .method(query.method())
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = test::read_body(resp).await;
        let resolve_struct: ResolveGidResp = serde_json::from_slice(&body).unwrap();
        println!("{:#?}", gid_struct);
        assert_eq!(resolve_struct.subnet_prefix, real_subnet_prefix);
        assert_eq!(resolve_struct.interface_id, real_interface_id);

        let networks = state.networks.lock().unwrap();
        let network = networks.get(&network_uuid).unwrap();
        let dev = &network.devices.iter().next().unwrap();
        println!("{:#?}", dev);

        let gid = &dev.gid[0];
        assert_eq!(dev.guid, Some(Virt::<u64>{real: GUID, virt: guid_struct.guid}));
        assert_eq!(gid, &Virt{
            virt: GidEntry{
                port: 1,
                idx: 0,
                subnet_prefix: gid_struct.subnet_prefix,
                interface_id: gid_struct.interface_id,
            },
            real: GidEntry{
                port: 1,
                idx: 0,
                subnet_prefix: real_subnet_prefix,
                interface_id: real_interface_id,
            }});
    }

    #[actix_rt::test]
    async fn build_put_gids() {
        let state = new_app_state();
        let mut app = test::init_service(
            App::new()
                .app_data(state.clone())
                .configure(config)).await;
        let network_uuid = Uuid::new_v4();
        let device_uuid = Uuid::new_v4();

        let query = LeaseDeviceQuery{
            guid: GUID,
        };
        let uri = query.compile(None, network_uuid, Some(device_uuid));
        println!("{}", uri);
        let req = TestRequest::with_uri(&uri)
            .method(query.method())
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), StatusCode::CREATED);
        let body = test::read_body(resp).await;
        let guid_struct: LeaseDeviceResp = serde_json::from_slice(&body).unwrap();
        assert_ne!(GUID, guid_struct.guid);

        let query = SetGidQuery{
            virt_port: 1,
            virt_idx: 0,
            virt_subnet_prefix: 10,
            virt_interface_id: 11,
            real_port: 1,
            real_idx: 0,
            real_subnet_prefix: 12,
            real_interface_id: 13,
        };
        let uri = query.compile(None, network_uuid, Some(device_uuid));
        let req = TestRequest::with_uri(&uri)
            .method(query.method())
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = test::read_body(resp).await;
        let resp: SetGidResp = serde_json::from_slice(&body).unwrap();
        println!("{:#?}", resp);

        let query = SetGidQuery{
            virt_port: 1,
            virt_idx: 1,
            virt_subnet_prefix: 0,
            virt_interface_id: 14,
            real_port: 1,
            real_idx: 1,
            real_subnet_prefix: 0,
            real_interface_id: 15,
        };
        let uri = query.compile(None, network_uuid, Some(device_uuid));
        let req = TestRequest::with_uri(&uri)
            .method(query.method())
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = test::read_body(resp).await;
        let resp: SetGidResp = serde_json::from_slice(&body).unwrap();
        println!("{:#?}", resp);

        let query = ResolveGidQuery{
            subnet_prefix: 0,
            interface_id: 14,
        };
        let uri = query.compile(None, network_uuid, None);
        println!("{}", uri);
        let req = TestRequest::with_uri(&uri)
            .method(query.method())
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = test::read_body(resp).await;
        let resolve_struct: ResolveGidResp = serde_json::from_slice(&body).unwrap();
        assert_eq!(resolve_struct.subnet_prefix, 0);
        assert_eq!(resolve_struct.interface_id, 15);

        let networks = state.networks.lock().unwrap();
        let network = networks.get(&network_uuid).unwrap();
        let dev = &network.devices.iter().next().unwrap();
        println!("{:#?}", dev);

        assert_eq!(dev.guid, Some(Virt::<u64>{real: GUID, virt: guid_struct.guid}));
        assert_eq!(dev.gid[1].virt.is_same_addr(&GidEntry{
            port: 144,
            idx: 44,
            subnet_prefix: 0,
            interface_id: 14,
        }), true);
        assert_eq!(dev.gid[1].real,
                   GidEntry{
                       port: 1,
                       idx: 1,
                       subnet_prefix: 0,
                       interface_id: 15,
                   })
    }
}
