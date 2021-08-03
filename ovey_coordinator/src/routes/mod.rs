//! Handles all routes/controllers. Functions that get invoked on a specific route from
//! Ovey daemon requests.

use actix_web::web;

mod types;
mod guids;
mod gids;
mod network;
mod ports;

use types::*;

pub(crate) fn new_app_state() -> web::Data<CoordState> {
    web::Data::new(CoordState::new())
}

pub(crate) fn config(cfg: &mut web::ServiceConfig) {
    guids::config(cfg);
    gids::config(cfg);
    network::config(cfg);
    ports::config(cfg);
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    use serde::de::DeserializeOwned;
    use actix_web::{test, App};
    use actix_http::Request;
    use actix_web::test::TestRequest;
    use actix_web::http::StatusCode;
    use actix_web::dev::{Service, MessageBody};
    use actix_web::dev::ServiceResponse;
    use liboveyutil::types::*;

    const GUID: u64 = 444;

    async fn do_request<'a, Q, S, B, E, R>(
        app: &mut S, network_uuid: Uuid, device_uuid: Option<Uuid>,
        port: Option<u32>, query: &Q, status: StatusCode) -> R
    where
        Q: OveydQuery,
        S: Service<Request = Request, Response = ServiceResponse<B>, Error = E>,
        E: std::fmt::Debug,
        B: MessageBody + Unpin,
        R: DeserializeOwned
    {
        let uri = query.compile(None, network_uuid, device_uuid, port);
        let req : Request = TestRequest::with_uri(&uri)
            .method(query.method())
            .to_request();
        let resp = test::call_service(app, req).await;
        assert_eq!(resp.status(), status);
        let body = test::read_body(resp).await;
        serde_json::from_slice(&body).unwrap()
    }

    async fn do_request_port<'a, Q, S, B, E, R>(
        app: &mut S,
        network_uuid: Uuid, device_uuid: Uuid, port: u32,
        query: &Q, status: StatusCode) -> R
    where
        Q: OveydQuery,
        S: Service<Request = Request, Response = ServiceResponse<B>, Error = E>,
        E: std::fmt::Debug,
        B: MessageBody + Unpin,
        R: DeserializeOwned
    {
        do_request(app, network_uuid, Some(device_uuid), Some(port), query, status).await
    }

    async fn do_request_device<'a, Q, S, B, E, R>(
        app: &mut S, network_uuid: Uuid, device_uuid: Uuid, query: &Q, status: StatusCode) -> R
    where
        Q: OveydQuery,
        S: Service<Request = Request, Response = ServiceResponse<B>, Error = E>,
        E: std::fmt::Debug,
        B: MessageBody + Unpin,
        R: DeserializeOwned
    {
        do_request(app, network_uuid, Some(device_uuid), None, query, status).await
    }

    async fn do_request_network<'a, Q, S, B, E, R>(
        app: &mut S, network_uuid: Uuid, query: &Q, status: StatusCode) -> R
    where
        Q: OveydQuery,
        S: Service<Request = Request, Response = ServiceResponse<B>, Error = E>,
        E: std::fmt::Debug,
        B: MessageBody + Unpin,
        R: DeserializeOwned
    {
        do_request(app, network_uuid, None, None, query, status).await
    }

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
        let resp1_struct: LeaseDeviceResp =
            do_request_device(&mut app, network, device,
                              &query, StatusCode::CREATED).await;
        assert_ne!(GUID, resp1_struct.guid);

        let resp2_struct: LeaseDeviceResp =
            do_request_device(&mut app, network, device,
                              &query, StatusCode::OK).await;
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
        let guid_struct: LeaseDeviceResp =
            do_request_device(&mut app,
                              network_uuid, device_uuid,
                              &query, StatusCode::CREATED).await;
        assert_ne!(GUID, guid_struct.guid);

        let query = CreatePortQuery{
            port: 1,
	          pkey_tbl_len: 16,
	          gid_tbl_len: 16,
	          core_cap_flags: 0,
	          max_mad_size: 16,
        };
        let _created: CreatePortResp = do_request_device(&mut app,
                                                        network_uuid, device_uuid,
                                                        &query, StatusCode::CREATED).await;

        let query = LeaseGidQuery{
            port: 1,
            idx: 0,
            subnet_prefix: real_subnet_prefix,
            interface_id: real_interface_id,
        };
        let gid_struct: LeaseGidResp =
            do_request_device(&mut app,
                              network_uuid, device_uuid,
                              &query, StatusCode::OK).await;
        println!("{:#?}", gid_struct);
        assert_eq!(gid_struct.port, query.port);
        assert_eq!(gid_struct.idx, query.idx);
        assert_ne!(gid_struct.subnet_prefix, query.subnet_prefix);
        assert_ne!(gid_struct.interface_id, query.interface_id);

        let query = ResolveGidQuery{
            subnet_prefix: gid_struct.subnet_prefix,
            interface_id: gid_struct.interface_id,
        };
        let resolve_struct: ResolveGidResp =
            do_request_network(&mut app,
                               network_uuid,
                               &query, StatusCode::OK).await;
        println!("{:#?}", resolve_struct);
        assert_eq!(resolve_struct.subnet_prefix, real_subnet_prefix);
        assert_eq!(resolve_struct.interface_id, real_interface_id);

        state.with_network(network_uuid, |network| {
            let dev = &network.devices.iter().next().unwrap();
            println!("{:#?}", dev);

            let gid = dev.ports[0].iter_gid().next().unwrap();
            assert_eq!(dev.guid, Some(Virt::<u64>{real: GUID, virt: guid_struct.guid}));
            assert_eq!(gid, &Virt{
                virt: GidEntry{
                    idx: 0,
                    subnet_prefix: gid_struct.subnet_prefix,
                    interface_id: gid_struct.interface_id,
                },
                real: GidEntry{
                    idx: 0,
                    subnet_prefix: real_subnet_prefix,
                    interface_id: real_interface_id,
                }});
            Ok(())
        }).unwrap();
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
        let guid_struct: LeaseDeviceResp =
            do_request_device(&mut app, network_uuid, device_uuid,
                              &query, StatusCode::CREATED).await;
        assert_ne!(GUID, guid_struct.guid);

        let query = CreatePortQuery{
            port: 1,
	          pkey_tbl_len: 16,
	          gid_tbl_len: 16,
	          core_cap_flags: 0,
	          max_mad_size: 16,
        };
        let _created: CreatePortResp = do_request_device(&mut app,
                                                        network_uuid, device_uuid,
                                                        &query, StatusCode::CREATED).await;

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
        let _resp: SetGidResp = do_request_device(&mut app,
                                                  network_uuid, device_uuid,
                                                  &query, StatusCode::OK).await;

        let query = SetGidQuery{
            virt_idx: 1,
            virt_subnet_prefix: 0,
            virt_interface_id: 14,
            real_idx: 1,
            real_subnet_prefix: 0,
            real_interface_id: 15,
        };
        let resp: SetGidResp = do_request_device(&mut app,
                                                 network_uuid, device_uuid,
                                                 &query, StatusCode::OK).await;
        println!("{:#?}", resp);

        let query = ResolveGidQuery{
            subnet_prefix: 0,
            interface_id: 14,
        };
        let resolve_struct: ResolveGidResp =
            do_request_network(&mut app,
                               network_uuid,
                               &query, StatusCode::OK).await;
        assert_eq!(resolve_struct.subnet_prefix, 0);
        assert_eq!(resolve_struct.interface_id, 15);

        state.with_network(network_uuid, |network| {
            let dev = &network.devices.iter().next().unwrap();
            println!("{:#?}", dev);

            assert_eq!(dev.guid, Some(Virt::<u64>{real: GUID, virt: guid_struct.guid}));
            assert_eq!(dev.ports[0]
                       .iter_gid().take(2).last().unwrap()
                       .virt.is_same_addr(&GidEntry{
                idx: 44,
                subnet_prefix: 0,
                interface_id: 14,
            }), true);
            assert_eq!(dev.ports[0]
                       .iter_gid().take(2).last().unwrap()
                       .real,
                       GidEntry{
                           idx: 1,
                           subnet_prefix: 0,
                           interface_id: 15,
                       });
            Ok(())
        }).unwrap();
    }
}
