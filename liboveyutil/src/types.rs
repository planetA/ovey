//! Common Types used in Ovey.
use serde::{Deserialize, Serialize};

/// A guid is a big endian encoded u64.
pub type GuidInternalType = u64;
/// Virtual GUID as String (e.g. dead:beef:affe:cafe) is the key.
/// This is easier to read/write during development and overhead is neglible.
pub type GuidString = String;
/// Virtual LID as String (e.g. 0x41) is the key.
/// This is easier to read/write during development and overhead is neglible.
pub type LidString = String;

#[derive(Debug, Serialize, Deserialize)]
pub struct LeaseDeviceReq {
    pub guid: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LeaseDeviceResp {
    pub guid: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LeaseGidReq {
    pub port: u16,
    pub idx: u32,
    pub subnet_prefix: u64,
    pub interface_id: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LeaseGidResp {
    pub port: u16,
    pub idx: u32,
    pub subnet_prefix: u64,
    pub interface_id: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResolveGidReq {
    pub subnet_prefix: u64,
    pub interface_id: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResolveGidResp {
    pub subnet_prefix: u64,
    pub interface_id: u64,
}
