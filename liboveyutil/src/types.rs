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
use crate::urls::*;

#[derive(Debug)]
pub struct OveydReq {
    pub seq: u32,
    pub network: Uuid,
    pub device: Option<Uuid>,
    pub query: Box<dyn OveydQuery>,
}

#[derive(Debug)]
pub enum OveydCmdResp {
    LeaseDevice(LeaseDeviceResp),
    LeaseGid(LeaseGidResp),
    ResolveGid(ResolveGidResp),
    SetGid(SetGidResp),
    CreatePort(CreatePortResp),
}

pub struct OveydResp {
    pub seq: u32,
    pub network: Uuid,
    pub cmd: OveydCmdResp,
}

pub trait OveydQuery: fmt::Debug {
    fn method(&self) -> http::Method;

    /// Endpoint at the coordinator that processes the query
    fn endpoint(&self) -> &str;

    /// Convert the query to urlencoded string
    fn query(&self) -> String;

    fn compile(&self, host: Option<&str>, network: Uuid, device: Option<Uuid>) -> String {
        let url = if let Some(device_uuid) = device {
            build_device_url(self.endpoint(), network, device_uuid)
        } else {
            build_network_url(self.endpoint(), network)
        };
        if let Some(host) = host {
            format!("{}{}?{}", host, url, self.query())
        } else {
            format!("{}?{}", url, self.query())
        }
    }

    fn parse_response(&self, res: String) -> Result<OveydCmdResp, std::io::Error>;
}


#[derive(Serialize, Deserialize, Debug)]
pub struct LeaseDeviceQuery {
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

    fn endpoint(&self) -> &str {
        ROUTE_GUIDS_DEVICE
    }

    fn query(&self) -> String {
        serde_urlencoded::to_string(&self).unwrap()
    }

    fn parse_response(&self, res: String) -> Result<OveydCmdResp, std::io::Error> {
        Ok(OveydCmdResp::LeaseDevice(serde_json::from_str::<LeaseDeviceResp>(&res)?))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LeaseGidQuery {
    pub port: u16,
    pub idx: u32,
    pub subnet_prefix: u64,
    pub interface_id: u64,
}

impl OveydQuery for LeaseGidQuery {
    fn method(&self) -> http::Method {
        http::Method::POST
    }

    fn endpoint(&self) -> &str {
        ROUTE_GIDS_DEVICE
    }

    fn query(&self) -> String {
        serde_urlencoded::to_string(&self).unwrap()
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
    pub subnet_prefix: u64,
    pub interface_id: u64,
}

impl OveydQuery for ResolveGidQuery {
    fn method(&self) -> http::Method {
        http::Method::GET
    }

    fn endpoint(&self) -> &str {
        ROUTE_GIDS_ALL
    }

    fn query(&self) -> String {
        serde_urlencoded::to_string(&self).unwrap()
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

    fn endpoint(&self) -> &str {
        ROUTE_GIDS_DEVICE
    }

    fn query(&self) -> String {
        serde_urlencoded::to_string(&self).unwrap()
    }

    fn parse_response(&self, res: String) -> Result<OveydCmdResp, std::io::Error> {
        Ok(OveydCmdResp::SetGid(
            serde_json::from_str::<SetGidResp>(&res)?))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetGidResp {
    pub real_port: u16,
    pub virt_port: u16,
    pub real_idx: u32,
    pub virt_idx: u32,
    pub real_subnet_prefix: u64,
    pub real_interface_id: u64,
    pub virt_subnet_prefix: u64,
    pub virt_interface_id: u64,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePortQuery {
    pub port: u16,
	  pub pkey_tbl_len: u32,
	  pub gid_tbl_len: u32,
	  pub core_cap_flags: u32,
	  pub max_mad_size: u32,

}

impl OveydQuery for CreatePortQuery {
    fn method(&self) -> http::Method {
        http::Method::POST
    }

    fn endpoint(&self) -> &str {
        ROUTE_PORTS_DEVICE
    }

    fn query(&self) -> String {
        serde_urlencoded::to_string(&self).unwrap()
    }

    fn parse_response(&self, res: String) -> Result<OveydCmdResp, std::io::Error> {
        Ok(OveydCmdResp::CreatePort(
            serde_json::from_str::<CreatePortResp>(&res)?))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePortResp {
    pub port: u16,
	  pub pkey_tbl_len: u32,
	  pub gid_tbl_len: u32,
	  pub core_cap_flags: u32,
	  pub max_mad_size: u32,
}
