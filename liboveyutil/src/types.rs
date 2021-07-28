//! Common Types used in Ovey.
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::fmt;
use serde_urlencoded;
use http;

/// A guid is a big endian encoded u64.
pub type GuidInternalType = u64;
/// Virtual GUID as String (e.g. dead:beef:affe:cafe) is the key.
/// This is easier to read/write during development and overhead is neglible.
pub type GuidString = String;
/// Virtual LID as String (e.g. 0x41) is the key.
/// This is easier to read/write during development and overhead is neglible.
pub type LidString = String;

#[derive(Debug)]
pub struct OveydReq {
    pub seq: u32,
    pub network: Uuid,
    pub query: Box<dyn OveydQuery>,
}

#[derive(Debug)]
pub enum OveydCmdResp {
    LeaseDevice(LeaseDeviceResp),
    LeaseGid(LeaseGidResp),
    ResolveGid(ResolveGidResp),
    SetGid(SetGidResp),
}

pub struct OveydResp {
    pub seq: u32,
    pub network: Uuid,
    pub cmd: OveydCmdResp,
}

pub trait OveydQuery: fmt::Debug {
    fn method(&self) -> http::Method;

    fn query(&self, network: Uuid) -> String;

    fn parse_response(&self, res: String) -> Result<OveydCmdResp, std::io::Error>;
}


#[derive(Serialize, Deserialize, Debug)]
pub struct LeaseDeviceQuery {
    pub device: Uuid,
    pub guid: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LeaseDeviceResp {
    pub guid: u64,
}

impl OveydQuery for LeaseDeviceQuery {
    fn method(&self) -> http::Method {
        http::Method::POST
    }

    fn query(&self, network: Uuid) -> String {
        let query = serde_urlencoded::to_string(&self).unwrap();
        format!("/net/{}/guid?{}", network, query)
    }

    fn parse_response(&self, res: String) -> Result<OveydCmdResp, std::io::Error> {
        Ok(OveydCmdResp::LeaseDevice(serde_json::from_str::<LeaseDeviceResp>(&res)?))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LeaseGidQuery {
    pub device: Uuid,
    pub port: u16,
    pub idx: u32,
    pub subnet_prefix: u64,
    pub interface_id: u64,
}

impl OveydQuery for LeaseGidQuery {
    fn method(&self) -> http::Method {
        http::Method::POST
    }

    fn query(&self, network: Uuid) -> String {
        let query = serde_urlencoded::to_string(&self).unwrap();
        format!("/net/{}/gid?{}", network, query)
    }

    fn parse_response(&self, res: String) -> Result<OveydCmdResp, std::io::Error> {
        Ok(OveydCmdResp::LeaseGid(
            serde_json::from_str::<LeaseGidResp>(&res)?))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LeaseGidResp {
    pub port: u16,
    pub idx: u32,
    pub subnet_prefix: u64,
    pub interface_id: u64,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct ResolveGidQuery {
    pub device: Uuid,
    pub subnet_prefix: u64,
    pub interface_id: u64,
}

impl OveydQuery for ResolveGidQuery {
    fn method(&self) -> http::Method {
        http::Method::GET
    }
    fn query(&self, network: Uuid) -> String {
        let query = serde_urlencoded::to_string(&self).unwrap();
        format!("/net/{}/gid?{}", network, query)
    }

    fn parse_response(&self, res: String) -> Result<OveydCmdResp, std::io::Error> {
        Ok(OveydCmdResp::ResolveGid(
            serde_json::from_str::<ResolveGidResp>(&res)?))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResolveGidResp {
    pub subnet_prefix: u64,
    pub interface_id: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetGidQuery {
    pub device: Uuid,
    pub real_port: u16,
    pub virt_port: u16,
    pub real_idx: u32,
    pub virt_idx: u32,
    pub virt_subnet_prefix: u64,
    pub virt_interface_id: u64,
    pub real_subnet_prefix: u64,
    pub real_interface_id: u64,
}

impl OveydQuery for SetGidQuery {
    fn method(&self) -> http::Method {
        http::Method::PUT
    }

    fn query(&self, network: Uuid) -> String {
        let query = serde_urlencoded::to_string(&self).unwrap();
        format!("/net/{}/gid?{}", network, query)
    }

    fn parse_response(&self, res: String) -> Result<OveydCmdResp, std::io::Error> {
        Ok(OveydCmdResp::SetGid(
            serde_json::from_str::<SetGidResp>(&res)?))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetGidResp {
    pub device: Uuid,
    pub real_port: u16,
    pub virt_port: u16,
    pub real_idx: u32,
    pub virt_idx: u32,
    pub real_subnet_prefix: u64,
    pub real_interface_id: u64,
    pub virt_subnet_prefix: u64,
    pub virt_interface_id: u64,
}
