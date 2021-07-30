//! Handles all routes/controllers. Functions that get invoked on a specific route from
//! Ovey daemon requests.

use actix_web::web;

use std::collections::HashMap;
use std::sync::Mutex;

mod types;
mod guids;
mod gids;
mod network;

use types::*;

pub(crate) fn new_app_state() -> web::Data<CoordState> {
    web::Data::new(CoordState{
        networks: Mutex::new(HashMap::new())
    })
}

pub(crate) fn config(cfg: &mut web::ServiceConfig) {
    guids::config(cfg);
    gids::config(cfg);
    network::config(cfg);
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    use actix_web::{test, App};
    use actix_web::test::TestRequest;
    use actix_web::http::StatusCode;
    use liboveyutil::types::*;

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
