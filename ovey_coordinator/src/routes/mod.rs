//! Handles all routes/controllers. Functions that get invoked on a specific route from
//! Ovey daemon requests.

use actix_web::web;

mod types;
mod guids;
mod gids;
mod network;
mod ports;
mod qp;

use types::*;

pub(crate) fn new_app_state() -> web::Data<CoordState> {
    web::Data::new(CoordState::new())
}

pub(crate) fn config(cfg: &mut web::ServiceConfig) {
    guids::config(cfg);
    gids::config(cfg);
    network::config(cfg);
    ports::config(cfg);
    qp::config(cfg);
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    use serde::de::DeserializeOwned;
    use serde::{Serialize};
    use actix_web::{test, App};
    use actix_http::Request;
    use actix_web::test::TestRequest;
    use actix_web::http::StatusCode;
    use actix_web::dev::{Service, MessageBody};
    use actix_web::dev::ServiceResponse;
    use liboveyutil::types::*;
    use rand::prelude::*;
    use serde_json::Result;
    use gids::DEFULT_GID_PREFIX;
    use guids::{OVEY_GUID_PREFIX_UMASK, OVEY_GUID_PREFIX};

    const GUID: u64 = 444;

    async fn do_request<'a, Q, S, B, E, R>(
        app: &mut S, network_uuid: Uuid, device_uuid: Option<Uuid>,
        port: Option<u16>, query: &Q, status: StatusCode) -> Result<R>
    where
        Q: OveydQuery + Serialize,
        S: Service<Request = Request, Response = ServiceResponse<B>, Error = E>,
        E: std::fmt::Debug,
        B: MessageBody + Unpin,
        R: DeserializeOwned
    {
        let uri = query.compile(None, network_uuid, device_uuid, port);
        println!("{}", uri);
        let req : Request = TestRequest::with_uri(&uri)
            .method(query.method())
            .header("content-type", "application/json")
            .set_payload(query.json())
            .to_request();
        println!("{:#?}", req);
        println!("{:#?}", query.json());
        let resp = test::call_service(app, req).await;
        let resp_status = resp.status();
        println!("{:#?}", resp);
        let body = test::read_body(resp).await;
        println!("{:?}", String::from_utf8((&body).to_vec()));
        assert_eq!(resp_status, status);
        serde_json::from_slice(&body)
    }

    async fn do_request_port<'a, Q, S, B, E, R>(
        app: &mut S,
        network_uuid: Uuid, device_uuid: Uuid, port: u16,
        query: &Q, status: StatusCode) -> Result<R>
    where
        Q: OveydQuery + Serialize,
        S: Service<Request = Request, Response = ServiceResponse<B>, Error = E>,
        E: std::fmt::Debug,
        B: MessageBody + Unpin,
        R: DeserializeOwned
    {
        do_request(app, network_uuid, Some(device_uuid), Some(port), query, status).await
    }

    async fn do_request_device<'a, Q, S, B, E, R>(
        app: &mut S, network_uuid: Uuid, device_uuid: Uuid,
        query: &Q, status: StatusCode) -> Result<R>
    where
        Q: OveydQuery + Serialize,
        S: Service<Request = Request, Response = ServiceResponse<B>, Error = E>,
        E: std::fmt::Debug,
        B: MessageBody + Unpin,
        R: DeserializeOwned
    {
        do_request(app, network_uuid, Some(device_uuid), None, query, status).await
    }

    async fn do_request_network<'a, Q, S, B, E, R>(
        app: &mut S, network_uuid: Uuid, query: &Q,
        status: StatusCode) -> Result<R>
    where
        Q: OveydQuery + Serialize,
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
                              &query, StatusCode::CREATED).await.unwrap();
        assert_ne!(GUID, resp1_struct.guid);
        println!("GUID {:#08x}", resp1_struct.guid);
        assert_eq!(resp1_struct.guid & !OVEY_GUID_PREFIX_UMASK, OVEY_GUID_PREFIX);

        let resp2_struct: LeaseDeviceResp =
            do_request_device(&mut app, network, device,
                              &query, StatusCode::OK).await.unwrap();
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
        let real = Gid{
            subnet_prefix: DEFULT_GID_PREFIX,
            interface_id: 5,
        };

        let query = LeaseDeviceQuery{
            guid: GUID,
        };
        let guid_struct: LeaseDeviceResp =
            do_request_device(&mut app,
                              network_uuid, device_uuid,
                              &query, StatusCode::CREATED).await.unwrap();
        assert_ne!(GUID, guid_struct.guid);

        let query = CreatePortQuery{
            port: 1,
	          pkey_tbl_len: 16,
	          gid_tbl_len: 16,
	          core_cap_flags: 0,
	          max_mad_size: 16,
        };
        let port: CreatePortResp =
            do_request_device(&mut app,
                              network_uuid, device_uuid,
                              &query, StatusCode::CREATED).await.unwrap();

        let query = SetPortAttrQuery{
            lid: 0,
        };
        let attr1: SetPortAttrResp =
            do_request_port(&mut app,
                            network_uuid, device_uuid, port.port,
                            &query, StatusCode::OK).await.unwrap();
        println!("{:#?}", attr1);
        assert_ne!(attr1.lid, 0);

        let query = SetPortAttrQuery{
            lid: 0,
        };
        let attr2: SetPortAttrResp =
            do_request_port(&mut app,
                            network_uuid, device_uuid, port.port,
                            &query, StatusCode::OK).await.unwrap();
        println!("{:#?}", attr2);
        assert_ne!(attr2.lid, 0);
        assert_eq!(attr2.lid, attr1.lid);

        let query = SetPortAttrQuery{
            lid: 1,
        };
        let attr: SetPortAttrResp =
            do_request_port(&mut app,
                            network_uuid, device_uuid, port.port,
                            &query, StatusCode::OK).await.unwrap();
        println!("{:#?}", attr);
        assert_ne!(attr.lid, 0);
        assert_ne!(attr2.lid, attr.lid);
        assert_eq!(attr.lid < (1 << 16), true);
        assert_eq!(attr2.lid < (1 << 16), true);

        let query = LeaseGidQuery{
            idx: 0,
            gid: real,
        };
        let gid_struct: LeaseGidResp =
            do_request_port(&mut app,
                            network_uuid, device_uuid, port.port,
                            &query, StatusCode::OK).await.unwrap();
        println!("{:#?}", gid_struct);
        assert_eq!(gid_struct.idx, query.idx);
        assert_ne!(gid_struct.gid.interface_id, query.gid.interface_id);

        let qp_query = CreateQpQuery{
            qpn: 42,
        };
        let qp_resp: CreateQpResp =
            do_request_device(&mut app,
                              network_uuid, device_uuid,
                              &qp_query, StatusCode::CREATED).await.unwrap();
        assert_ne!(qp_resp.qpn, qp_query.qpn);

        let query = ResolveQpGidQuery{
            qpn: qp_resp.qpn,
            gid: gid_struct.gid,
        };
        let resolve_struct: ResolveQpGidResp =
            do_request_network(&mut app,
                               network_uuid,
                               &query, StatusCode::OK).await.unwrap();
        println!("{:#?}", resolve_struct);
        assert_eq!(resolve_struct.gid.subnet_prefix, real.subnet_prefix);
        assert_eq!(resolve_struct.gid.interface_id, real.interface_id);
        assert_eq!(resolve_struct.qpn, qp_query.qpn);

        state.with_network(network_uuid, |network| {
            let dev = &network.devices.iter().next().unwrap();
            println!("{:#?}", dev);

            let gid = dev.iter_port().next().unwrap().iter_gid().next().unwrap();
            assert_eq!(dev.guid, Some(Virt::<u64>{real: GUID, virt: guid_struct.guid}));
            assert_eq!(gid, &Virt{
                virt: GidEntry{
                    idx: 0,
                    gid: gid_struct.gid,
                },
                real: GidEntry{
                    idx: 0,
                    gid: real,
                }});
            Ok(())
        }).unwrap();

        let query = SetGidQuery {
            real_idx: 0,
            virt_idx: gid_struct.idx,
            real: real,
            virt: gid_struct.gid,
        };
        let put_resp: SetGidResp =
            do_request_port(&mut app,
                            network_uuid, device_uuid, port.port,
                            &query, StatusCode::OK).await.unwrap();
        println!("{:#?}", put_resp);
        assert_eq!(put_resp.virt_idx, query.virt_idx);
        assert_eq!(put_resp.real_idx, query.real_idx);
        assert_eq!(put_resp.real, query.real);
        assert_eq!(put_resp.virt, query.virt);

        let query = SetGidQuery {
            real_idx: 1,
            virt_idx: 15,
            real: real,
            virt: gid_struct.gid,
        };
        let err_resp: Result<SetGidResp> =
            do_request_port(&mut app,
                            network_uuid, device_uuid, port.port,
                            &query, StatusCode::CONFLICT).await;
        assert_eq!(err_resp.is_err(), true);

        let query = SetGidQuery {
            real_idx: 1,
            virt_idx: gid_struct.idx,
            real: Gid{
                subnet_prefix: DEFULT_GID_PREFIX,
                interface_id: 5,
            },
            virt: gid_struct.gid,
        };
        let gid_resp: SetGidResp =
            do_request_port(&mut app,
                            network_uuid, device_uuid, port.port,
                            &query, StatusCode::OK).await.unwrap();
        assert_eq!(gid_resp.virt_idx, query.virt_idx);
        assert_eq!(gid_resp.real_idx, query.real_idx);
        assert_eq!(gid_resp.real, query.real);
        assert_eq!(gid_resp.virt, query.virt);
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
                              &query, StatusCode::CREATED).await.unwrap();
        assert_ne!(GUID, guid_struct.guid);

        let query = CreatePortQuery{
            port: 1,
	          pkey_tbl_len: 16,
	          gid_tbl_len: 16,
	          core_cap_flags: 0,
	          max_mad_size: 16,
        };
        let port: CreatePortResp =
            do_request_device(&mut app,
                              network_uuid, device_uuid,
                              &query, StatusCode::CREATED).await.unwrap();

        let query = SetGidQuery{
            virt_idx: 0,
            real_idx: 0,
            virt: Gid{
                subnet_prefix: DEFULT_GID_PREFIX,
                interface_id: 11,
            },
            real: Gid{
                subnet_prefix: 12,
                interface_id: 13,
            },
        };
        let resp: SetGidResp =
            do_request_port(&mut app,
                            network_uuid, device_uuid, port.port,
                            &query, StatusCode::OK).await.unwrap();
        assert_eq!(resp.virt, query.virt);
        assert_eq!(resp.real, query.real);
        assert_ne!(resp.real, query.virt);

        let query = SetGidQuery{
            virt_idx: 1,
            real_idx: 1,
            virt: Gid{
                subnet_prefix: DEFULT_GID_PREFIX,
                interface_id: 14,
            },
            real: Gid{
                subnet_prefix: 0,
                interface_id: 15,
            },
        };
        let resp: SetGidResp =
            do_request_port(&mut app,
                            network_uuid, device_uuid, port.port,
                            &query, StatusCode::OK).await.unwrap();
        assert_eq!(resp.virt, query.virt);
        assert_eq!(resp.real, query.real);
        assert_ne!(resp.real, query.virt);

        let qp_query = CreateQpQuery{
            qpn: 42,
        };
        let qp_resp: CreateQpResp =
            do_request_device(&mut app,
                              network_uuid, device_uuid,
                              &qp_query, StatusCode::CREATED).await.unwrap();
        assert_ne!(qp_resp.qpn, qp_query.qpn);

        let query = ResolveQpGidQuery{
            gid: Gid{
                subnet_prefix: DEFULT_GID_PREFIX,
                interface_id: 14,
            },
            qpn: qp_resp.qpn,
        };
        let resolve_struct: ResolveQpGidResp =
            do_request_network(&mut app,
                               network_uuid,
                               &query, StatusCode::OK).await.unwrap();
        println!("{:#?}", resolve_struct);
        assert_eq!(resolve_struct.gid.subnet_prefix, 0);
        assert_eq!(resolve_struct.gid.interface_id, 15);
        assert_eq!(resolve_struct.qpn, qp_query.qpn);

        state.with_network(network_uuid, |network| {
            let dev = &network.devices.iter().next().unwrap();
            println!("{:#?}", dev);

            assert_eq!(dev.guid, Some(Virt::<u64>{real: GUID, virt: guid_struct.guid}));
            assert_eq!(dev.iter_port().next().unwrap()
                       .iter_gid().take(2).last().unwrap()
                       .virt.is_same_addr(&GidEntry{
                           idx: 44,
                           gid: Gid{
                               subnet_prefix: DEFULT_GID_PREFIX,
                               interface_id: 14,
                           },
            }), true);
            assert_eq!(dev.iter_port().next().unwrap()
                       .iter_gid().take(2).last().unwrap()
                       .real,
                       GidEntry{
                           idx: 1,
                           gid: Gid{
                               subnet_prefix: 0,
                               interface_id: 15,
                           },
                       });
            Ok(())
        }).unwrap();
    }

    #[actix_rt::test]
    async fn build_unique_gids() {
        let state = new_app_state();
        let mut app = test::init_service(
            App::new()
                .app_data(state.clone())
                .configure(config)).await;
        let network_uuid = Uuid::new_v4();
        let device_a = Uuid::new_v4();
        let device_b = Uuid::new_v4();
        let guid_a: u64 = random();
        let guid_b: u64 = random();

        let guid_resp_a: LeaseDeviceResp =
            do_request_device(&mut app, network_uuid, device_a,
                              &LeaseDeviceQuery{
                                  guid: guid_a,
                              },
                              StatusCode::CREATED).await.unwrap();
        assert_ne!(guid_resp_a.guid, guid_a);

        let port_resp_a: CreatePortResp =
            do_request_device(&mut app,
                              network_uuid, device_a,
                              &CreatePortQuery{
                                  port: 1,
                                  pkey_tbl_len: 16,
                                  gid_tbl_len: 16,
                                  core_cap_flags: 0,
                                  max_mad_size: 16,
                              }, StatusCode::CREATED).await.unwrap();

        let set_port_a = SetPortAttrQuery{
            lid: 0,
        };
        let set_port_resp_a: SetPortAttrResp =
            do_request_port(&mut app,
                            network_uuid, device_a, port_resp_a.port,
                            &set_port_a, StatusCode::OK).await.unwrap();
        assert_ne!(set_port_resp_a.lid, 0);

        let set_port_a = SetPortAttrQuery{
            lid: 0,
        };
        let set_port_resp_a: SetPortAttrResp =
            do_request_port(&mut app,
                            network_uuid, device_a, port_resp_a.port,
                            &set_port_a, StatusCode::OK).await.unwrap();
        assert_ne!(set_port_resp_a.lid, 0);

        let guid_resp_b: LeaseDeviceResp =
            do_request_device(&mut app, network_uuid, device_b,
                              &LeaseDeviceQuery{
                                  guid: guid_b,
                              },
                              StatusCode::CREATED).await.unwrap();
        assert_ne!(guid_resp_b.guid, guid_b);

        let port_faux_b = 1;
        let set_gid_b = SetGidQuery{
            virt_idx: 0,
            real_idx: 0,
            virt: Gid{
                subnet_prefix: DEFULT_GID_PREFIX,
                interface_id: random(),
            },
            real: Gid{
                subnet_prefix: random(),
                interface_id: random(),
            },
        };
        let err_resp: Result<SetGidResp> =
            do_request_port(&mut app,
                            network_uuid, device_b, port_faux_b,
                            &set_gid_b, StatusCode::NOT_FOUND).await;
        assert_eq!(err_resp.is_err(), true);

        let set_gid_a = SetGidQuery{
            virt_idx: 0,
            real_idx: 0,
            virt: Gid{
                subnet_prefix: DEFULT_GID_PREFIX,
                interface_id: random(),
            },
            real: Gid{
                subnet_prefix: random(),
                interface_id: random(),
            },
        };
        let set_gid_resp_a: SetGidResp =
            do_request_port(&mut app,
                            network_uuid, device_a, port_resp_a.port,
                            &set_gid_a, StatusCode::OK).await.unwrap();
        assert_eq!(set_gid_resp_a.virt, set_gid_a.virt);
        assert_eq!(set_gid_resp_a.real, set_gid_a.real);
        assert_ne!(set_gid_resp_a.real, set_gid_a.virt);

        let port_resp_b: CreatePortResp =
            do_request_device(&mut app,
                              network_uuid, device_b,
                              &CreatePortQuery{
                                  port: 1,
                                  pkey_tbl_len: 16,
                                  gid_tbl_len: 16,
                                  core_cap_flags: 0,
                                  max_mad_size: 16,
                              }, StatusCode::CREATED).await.unwrap();

        let set_port_b = SetPortAttrQuery{
            lid: random::<u16>().into(),
        };
        let set_port_resp_b: SetPortAttrResp =
            do_request_port(&mut app,
                            network_uuid, device_b, port_resp_b.port,
                            &set_port_b, StatusCode::OK).await.unwrap();
        assert_ne!(set_port_resp_b.lid, 0);
        assert_eq!(set_port_resp_b.lid < (1 << 16), true);
    }
}
