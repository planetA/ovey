//! Handles all routes/controllers. Functions that get invoked on a specific route from
//! Ovey daemon requests.

use actix_web::{HttpResponse, HttpRequest, web, post, get, put};
use actix_web::http::StatusCode;

use crate::config::CONFIG;
use crate::rest::errors::CoordinatorRestError;
use uuid::Uuid;
use liboveyutil::types::*;
use std::collections::HashMap;
use std::hash::{Hasher, Hash};
use std::sync::Mutex;
use std::time::Instant;
use rand::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
struct Virt<T> {
    real: T,
    virt: T,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct GidEntry {
    port: u16,
    idx: u32,
    subnet_prefix: u64,
    interface_id: u64,
}

impl Hash for GidEntry {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.subnet_prefix.hash(state);
        self.interface_id.hash(state);
    }
}

impl GidEntry {
    fn new(idx: u32, subnet_prefix: u64, interface_id: u64) -> Self {
        GidEntry{
            port: 1,
            idx: idx,
            subnet_prefix,
            interface_id,
        }
    }
}

#[derive(Clone, Debug)]
struct DeviceEntry {
    device: Uuid,
    guid: Option<Virt<u64>>,
    gid: HashMap<GidEntry,GidEntry>,
    lease: Instant,
}

impl DeviceEntry {
    fn new(device: Uuid) -> Self {
        Self{
            device: device,
            lease: Instant::now(),
            guid: None,
            gid: HashMap::new(),
        }
    }

    fn set_guid(&mut self, guid: Virt<u64>) -> &mut Self {
        self.guid = Some(guid);
        self
    }

    fn set_gid(&mut self, gid: Virt<GidEntry>) -> &mut Self {
        self.gid.insert(gid.virt, gid.real);
        self
    }
}

#[derive(Debug)]
struct DeviceTable(Vec<DeviceEntry>);

impl DeviceTable {
    fn new() -> Self {
        Self(Vec::new())
    }

    fn by_device(&mut self, dev: Uuid) -> Option<&mut DeviceEntry> {
        self.0.iter_mut().find(|e| e.device == dev)
    }

    fn find<F>(&mut self, func: F) -> Option<&DeviceEntry> where
        F: FnMut(&&DeviceEntry) -> bool {
        self.0.iter().find(func)
    }

    fn insert(&mut self, entry: DeviceEntry) {
        self.0.push(entry);
    }

    fn vec(&self) -> &Vec<DeviceEntry> {
        &self.0
    }
}

struct NetworkState {
    devices: DeviceTable,
}

impl NetworkState {
    fn new() -> Self {
        NetworkState{
            devices: DeviceTable::new(),
        }
    }
}

pub struct CoordState {
    networks: Mutex<HashMap<Uuid, NetworkState>>,
}

pub fn new_app_state() -> web::Data<CoordState> {
    web::Data::new(CoordState{
        networks: Mutex::new(HashMap::new())
    })
}

pub async fn route_index(_req: HttpRequest) -> HttpResponse {
    //println!("request: {:?}", &req);
    HttpResponse::Gone().finish() // <- send response
}

pub async fn route_get_network_info(web::Path(network_uuid): web::Path<Uuid>)
  -> Result<actix_web::HttpResponse, CoordinatorRestError> {
    Ok(HttpResponse::Gone().finish())
}

pub async fn route_get_device_info(web::Path((network_uuid, virt_dev_id)): web::Path<(Uuid, GuidString)>)
    -> Result<actix_web::HttpResponse, CoordinatorRestError> {
    Ok(HttpResponse::Gone().finish())
}

#[post("/net/{network}/guid")]
pub async fn route_guid_post(
    state: web::Data<CoordState>,
    web::Path(network_uuid): web::Path<Uuid>,
    web::Query(query): web::Query<LeaseDeviceQuery>,
    _req: HttpRequest) -> Result<actix_web::HttpResponse, CoordinatorRestError>
{
    let mut networks = state.networks.lock().unwrap();
    let network = networks.entry(network_uuid).or_insert(NetworkState::new());

    let (status, virt) = if let Some(mut device) = network.devices.by_device(query.device) {
        device.lease = Instant::now();
        (StatusCode::OK, device.guid.unwrap().virt)
    } else {
        let device = DeviceEntry::new(query.device)
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
}

#[post("/net/{network}/gid")]
pub async fn route_gid_post(
    state: web::Data<CoordState>,
    web::Path(network_uuid): web::Path<Uuid>,
    web::Query(query): web::Query<LeaseGidQuery>,
    _req: HttpRequest) -> Result<actix_web::HttpResponse, CoordinatorRestError>
{
    let mut networks = state.networks.lock().unwrap();
    let network = networks.get_mut(&network_uuid).unwrap();
    debug!("Lease gd: {}: {:#?} {:#?}", network_uuid, _req, query);

    if let Some(device) = network.devices.by_device(query.device) {
        // Find the next available index
        let idx = device.gid.iter()
            .map(|(virt, _)| virt.idx)
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

#[get("/net/{network}/gid")]
pub async fn route_resolve_gid(
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
    let gid = network.devices.0.iter()
        .filter_map(|device| device.gid.get(&search_pattern))
        .collect::<Vec<_>>();
    if gid.len() == 0 {
        return Ok(HttpResponse::NotFound().finish());
    }

    let output = ResolveGidResp{
        subnet_prefix: gid[0].subnet_prefix,
        interface_id: gid[0].interface_id,
    };
    debug!("Resolve gid: {}: {:#?} {:#?} -> {:#?}", network_uuid, _req,
           query, output);
    Ok(HttpResponse::Ok().json(output))
}

#[put("/net/{network}/gid")]
pub async fn route_gid_put(
    state: web::Data<CoordState>,
    web::Path(network_uuid): web::Path<Uuid>,
    web::Query(query): web::Query<SetGidQuery>,
    _req: HttpRequest) -> Result<actix_web::HttpResponse, CoordinatorRestError>
{
    let mut networks = state.networks.lock().unwrap();
    let network = networks.get_mut(&network_uuid).unwrap();
    debug!("Let gd: {}: {:#?} {:#?}", network_uuid, _req, query);

    if let Some(device) = network.devices.by_device(query.device) {
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
            device: device.device,
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

pub async fn route_delete_device(web::Path((network_uuid, virt_dev_id)): web::Path<(Uuid, GuidString)>)
    -> Result<actix_web::HttpResponse, CoordinatorRestError> {
    Ok(HttpResponse::Gone().finish())
}


#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App};
    use actix_web::test::TestRequest;
    use actix_web::http::{Method, StatusCode};

    const GUID: u64 = 444;

    #[actix_rt::test]
    async fn build_lease_device_request() {
        let state = new_app_state();
        let mut app = test::init_service(
            App::new()
                .app_data(state.clone())
                .service(route_guid_post)).await;
        let network = Uuid::new_v4();

        let query_struct = LeaseDeviceQuery{
            guid: GUID,
            device: Uuid::new_v4(),
        };
        let uri = query_struct.query(network);
        let req = TestRequest::with_uri(&uri)
            .method(Method::POST)
            .to_request();
        println!("{:#?}", req);
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), StatusCode::CREATED);
        let body = test::read_body(resp).await;
        let resp1_struct: LeaseDeviceResp = serde_json::from_slice(&body).unwrap();
        assert_ne!(GUID, resp1_struct.guid);

        let req = TestRequest::with_uri(&uri)
            .method(Method::POST)
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
                .service(route_guid_post)
                .service(route_gid_post)
                .service(route_resolve_gid)
        ).await;
        let network_uuid = Uuid::new_v4();
        let device_uuid = Uuid::new_v4();
        let real_subnet_prefix: u64 = 4;
        let real_interface_id: u64 = 5;

        let query_struct = LeaseDeviceQuery{
            guid: GUID,
            device: device_uuid,
        };
        let uri = query_struct.query(network_uuid);
        println!("{}", uri);
        let req = TestRequest::with_uri(&uri)
            .method(Method::POST)
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), StatusCode::CREATED);
        let body = test::read_body(resp).await;
        let guid_struct: LeaseDeviceResp = serde_json::from_slice(&body).unwrap();
        assert_ne!(GUID, guid_struct.guid);

        let query_struct = LeaseGidQuery{
            device: device_uuid,
            port: 1,
            idx: 0,
            subnet_prefix: real_subnet_prefix,
            interface_id: real_interface_id,
        };
        let uri = query_struct.query(network_uuid);
        let req = TestRequest::with_uri(&uri)
            .method(Method::POST)
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = test::read_body(resp).await;
        let gid_struct: LeaseGidResp = serde_json::from_slice(&body).unwrap();
        println!("{:#?}", gid_struct);
        assert_eq!(gid_struct.port, query_struct.port);
        assert_eq!(gid_struct.idx, query_struct.idx);
        assert_ne!(gid_struct.subnet_prefix, query_struct.subnet_prefix);
        assert_ne!(gid_struct.interface_id, query_struct.interface_id);

        let query_struct = ResolveGidQuery{
            device: device_uuid,
            subnet_prefix: gid_struct.subnet_prefix,
            interface_id: gid_struct.interface_id,
        };
        let uri = query_struct.query(network_uuid);
        println!("{}", uri);
        let req = TestRequest::with_uri(&uri)
            .method(Method::GET)
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
        let dev = &network.devices.vec()[0];
        println!("{:#?}", dev);

        assert_eq!(dev.device, query_struct.device);
        assert_eq!(dev.guid, Some(Virt::<u64>{real: GUID, virt: guid_struct.guid}));
        assert_eq!(dev.gid.get(&GidEntry{
            port: 1,
            idx: 0,
            subnet_prefix: gid_struct.subnet_prefix,
            interface_id: gid_struct.interface_id,
        }).unwrap(), &GidEntry{
            port: 1,
            idx: 0,
            subnet_prefix: real_subnet_prefix,
            interface_id: real_interface_id,
        });
}
}
