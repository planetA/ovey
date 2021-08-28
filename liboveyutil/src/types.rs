//! Common Types used in Ovey.
use std::fmt::Formatter;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::fmt;
use http;
use byteorder::{BigEndian, WriteBytesExt};

use crate::urls::*;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Gid {
    pub subnet_prefix: u64,
    pub interface_id: u64,
}

impl Gid {
    /// A reserved address has interface ID set to zero (4.1.1 6) and should not
    /// be used for addressing
    pub fn is_reserved(&self) -> bool {
        self.interface_id == 0
    }

    /// A loopback address has interface ID set to ::1 (4.1.1 6) and should not
    /// be used for virtualisation by the coordinator
    pub fn is_loopback(&self) -> bool {
        self.interface_id == 1
    }
}

impl std::fmt::Display for Gid {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        let mut wtr = vec![];
        wtr.write_u64::<BigEndian>(self.subnet_prefix).unwrap();
        wtr.write_u64::<BigEndian>(self.interface_id).unwrap();
        for (i, w) in wtr.iter().enumerate() {
            if i > 0 && (i % 4 == 0) {
                write!(f, ":")?;
            }
            write!(f, "{:01x}", w)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct OveydReq {
    pub seq: u32,
    pub network: Uuid,
    pub device: Option<Uuid>,
    pub port: Option<u16>,
    pub query: Box<dyn OveydQuery>,
}

#[derive(Debug)]
pub enum OveydCmdResp {
    LeaseDevice(LeaseDeviceResp),
    LeaseGid(LeaseGidResp),
    ResolveQpGid(ResolveQpGidResp),
    SetGid(SetGidResp),
    CreatePort(CreatePortResp),
    SetPortAttr(SetPortAttrResp),
    CreateQp(CreateQpResp),
}

#[derive(Debug)]
pub struct OveydResp {
    pub seq: u32,
    pub network: Uuid,
    pub cmd: OveydCmdResp,
}

pub trait OveydQuery: fmt::Debug {
    fn method(&self) -> http::Method;

    /// Endpoint at the coordinator that processes the query
    fn endpoint(&self) -> &str;

    /// Convert the query to json
    fn json(&self) -> String;

    fn compile(&self, host: Option<&str>, network: Uuid, device: Option<Uuid>, port: Option<u16>) -> String {
        let url = if let Some(port) = port {
            build_port_url(self.endpoint(), network, device.unwrap(), port)
        } else if let Some(device_uuid) = device {
            build_device_url(self.endpoint(), network, device_uuid)
        } else {
            build_network_url(self.endpoint(), network)
        };
        if let Some(host) = host {
            format!("{}{}", host, url)
        } else {
            format!("{}", url)
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

    fn json(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }

    fn parse_response(&self, res: String) -> Result<OveydCmdResp, std::io::Error> {
        Ok(OveydCmdResp::LeaseDevice(serde_json::from_str::<LeaseDeviceResp>(&res)?))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LeaseGidQuery {
    pub idx: u32,
    pub gid: Gid,
}

impl OveydQuery for LeaseGidQuery {
    fn method(&self) -> http::Method {
        http::Method::POST
    }

    fn endpoint(&self) -> &str {
        ROUTE_GIDS_PORT
    }

    fn json(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }

    fn parse_response(&self, res: String) -> Result<OveydCmdResp, std::io::Error> {
        Ok(OveydCmdResp::LeaseGid(
            serde_json::from_str::<LeaseGidResp>(&res)?))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LeaseGidResp {
    pub idx: u32,
    pub gid: Gid,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct ResolveQpGidQuery {
    pub gid: Gid,
    pub qpn: u32,
}

impl OveydQuery for ResolveQpGidQuery {
    fn method(&self) -> http::Method {
        http::Method::GET
    }

    fn endpoint(&self) -> &str {
        ROUTE_GIDS_ALL
    }

    fn json(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }

    fn parse_response(&self, res: String) -> Result<OveydCmdResp, std::io::Error> {
        Ok(OveydCmdResp::ResolveQpGid(
            serde_json::from_str::<ResolveQpGidResp>(&res)?))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResolveQpGidResp {
    pub gid: Gid,
    pub qpn: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetGidQuery {
    pub real_idx: u32,
    pub virt_idx: u32,
    pub virt: Gid,
    pub real: Gid,
}

impl OveydQuery for SetGidQuery {
    fn method(&self) -> http::Method {
        http::Method::PUT
    }

    fn endpoint(&self) -> &str {
        ROUTE_GIDS_PORT
    }

    fn json(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }

    fn parse_response(&self, res: String) -> Result<OveydCmdResp, std::io::Error> {
        Ok(OveydCmdResp::SetGid(
            serde_json::from_str::<SetGidResp>(&res)?))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetGidResp {
    pub virt_idx: u32,
    pub real_idx: u32,
    pub virt: Gid,
    pub real: Gid,
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

    fn json(&self) -> String {
        serde_json::to_string(&self).unwrap()
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

#[derive(Debug, Serialize, Deserialize)]
pub struct SetPortAttrQuery {
    pub lid: u32,
}

impl OveydQuery for SetPortAttrQuery {
    fn method(&self) -> http::Method {
        http::Method::POST
    }

    fn endpoint(&self) -> &str {
        ROUTE_PORTS_ONE
    }

    fn json(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }

    fn parse_response(&self, res: String) -> Result<OveydCmdResp, std::io::Error> {
        Ok(OveydCmdResp::SetPortAttr(
            serde_json::from_str::<SetPortAttrResp>(&res)?))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetPortAttrResp {
    pub lid: u32,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct CreateQpQuery {
    pub qpn: u32,
}

impl OveydQuery for CreateQpQuery {
    fn method(&self) -> http::Method {
        http::Method::POST
    }

    fn endpoint(&self) -> &str {
        ROUTE_QPS_DEVICE
    }

    fn json(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }

    fn parse_response(&self, res: String) -> Result<OveydCmdResp, std::io::Error> {
        Ok(OveydCmdResp::CreateQp(
            serde_json::from_str::<CreateQpResp>(&res)?))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateQpResp {
    pub qpn: u32,
}
