use std::io::Write;
use std::io::Read;
use config::CONFIG;
use std::{thread, time};
use std::sync::Arc;
use std::fs::{OpenOptions};
use simple_on_shutdown::on_shutdown_move;
use std::sync::atomic::{AtomicBool, Ordering};
use std::mem::size_of;
use std::io;
use std::convert::TryInto;
use std::convert::TryFrom;
use std::slice;
use std::mem;
use liboveyutil::types::*;
use reqwest::{Client, StatusCode};

mod config;
use uuid::Uuid;

#[macro_use]
extern crate log;

enum OveydRequestType {
    LeaseDevice = 0,
    LeaseGid = 1,
    ResolveGid = 2,
    SetGid = 3,
    CreatePort = 4,
    SetPortAttr = 5,
}

// Big endian u64 type
#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct U64Be(u64);

impl From<u64> for U64Be {
    fn from(val: u64) -> Self {
        Self(u64::to_be(val))
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct oveyd_lease_device {
    guid: U64Be,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct oveyd_lease_gid {
    idx: u32,
    subnet_prefix: U64Be,
    interface_id: U64Be,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct oveyd_set_gid {
    real_idx: u32,
    virt_idx: u32,
    real_subnet_prefix: U64Be,
    real_interface_id: U64Be,
    virt_subnet_prefix: U64Be,
    virt_interface_id: U64Be,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct oveyd_resolve_gid {
    subnet_prefix: U64Be,
    interface_id: U64Be,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct oveyd_create_port {
    port: u16,
	  pkey_tbl_len: u32,
	  gid_tbl_len: u32,
	  core_cap_flags: u32,
	  max_mad_size: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct oveyd_set_port_attr {
    lid: u32,
}

#[repr(C)]
union cmd_union {
    pub lease_device: oveyd_lease_device,
    pub lease_gid: oveyd_lease_gid,
    pub resolve_gid: oveyd_resolve_gid,
    pub set_gid: oveyd_set_gid,
    pub create_port: oveyd_create_port,
    pub set_port_attr: oveyd_set_port_attr,
}

#[repr(C)]
struct oveyd_req_pkt {
    pub cmd_type: u16,
    pub len: u16,
    pub seq: u32,
    pub network: [u8; 16],
    pub device: [u8; 16],
    pub port: u16,
    pub cmd: cmd_union,
}

#[repr(C)]
struct oveyd_resp_pkt {
    pub cmd_type: u16,
    pub len: u16,
    pub seq: u32,
    pub cmd: cmd_union,
}

impl TryFrom<u16> for OveydRequestType {
    type Error = ();

    fn try_from(v: u16) -> Result<Self, Self::Error> {
        match v {
            x if x == OveydRequestType::LeaseDevice as u16 => Ok(OveydRequestType::LeaseDevice),
            x if x == OveydRequestType::LeaseGid as u16 => Ok(OveydRequestType::LeaseGid),
            x if x == OveydRequestType::ResolveGid as u16 => Ok(OveydRequestType::ResolveGid),
            x if x == OveydRequestType::SetGid as u16 => Ok(OveydRequestType::SetGid),
            x if x == OveydRequestType::CreatePort as u16 => Ok(OveydRequestType::CreatePort),
            x if x == OveydRequestType::SetPortAttr as u16 => Ok(OveydRequestType::SetPortAttr),
            _ => Err(()),
        }
    }
}

fn parse_request_lease_device(req: oveyd_req_pkt) -> Result<OveydReq, io::Error> {
    let cmd: oveyd_lease_device = unsafe {
        req.cmd.lease_device
    };

    Ok(OveydReq{
        seq: req.seq,
        network: Uuid::from_bytes(req.network),
        device: Some(Uuid::from_bytes(req.device)),
        port: None,
        query: Box::new(LeaseDeviceQuery{
            guid: u64::from_be(cmd.guid.0),
        }),
    })
}

fn parse_request_lease_gid(req: oveyd_req_pkt) -> Result<OveydReq, io::Error> {
    let cmd: oveyd_lease_gid = unsafe {
        req.cmd.lease_gid
    };

    Ok(OveydReq{
        seq: req.seq,
        network: Uuid::from_bytes(req.network),
        device: Some(Uuid::from_bytes(req.device)),
        port: Some(req.port),
        query: Box::new(LeaseGidQuery{
            idx: cmd.idx,
            subnet_prefix: u64::from_be(cmd.subnet_prefix.0),
            interface_id: u64::from_be(cmd.interface_id.0),
        }),
    })
}

fn parse_request_resolve_gid(req: oveyd_req_pkt) -> Result<OveydReq, io::Error> {
    let cmd: oveyd_resolve_gid = unsafe {
        req.cmd.resolve_gid
    };

    Ok(OveydReq{
        seq: req.seq,
        network: Uuid::from_bytes(req.network),
        device: None,
        port: None,
        query: Box::new(ResolveGidQuery{
            subnet_prefix: u64::from_be(cmd.subnet_prefix.0),
            interface_id: u64::from_be(cmd.interface_id.0),
        }),
    })
}

fn parse_request_set_gid(req: oveyd_req_pkt) -> Result<OveydReq, io::Error> {
    let cmd: oveyd_set_gid = unsafe {
        req.cmd.set_gid
    };

    println!("parse_request_set_gid: {:?}", cmd);

    Ok(OveydReq{
        seq: req.seq,
        network: Uuid::from_bytes(req.network),
        device: Some(Uuid::from_bytes(req.device)),
        port: Some(req.port),
        query: Box::new(SetGidQuery{
            real_idx: cmd.real_idx,
            virt_idx: cmd.virt_idx,
            virt_subnet_prefix: u64::from_be(cmd.virt_subnet_prefix.0),
            virt_interface_id: u64::from_be(cmd.virt_interface_id.0),
            real_subnet_prefix: u64::from_be(cmd.real_subnet_prefix.0),
            real_interface_id: u64::from_be(cmd.real_interface_id.0),
        }),
    })
}

fn parse_request_create_port(req: oveyd_req_pkt) -> Result<OveydReq, io::Error> {
    let cmd: oveyd_create_port = unsafe {
        req.cmd.create_port
    };

    Ok(OveydReq{
        seq: req.seq,
        network: Uuid::from_bytes(req.network),
        device: Some(Uuid::from_bytes(req.device)),
        port: None,
        query: Box::new(CreatePortQuery{
            port: cmd.port,
	          pkey_tbl_len: cmd.pkey_tbl_len,
	          gid_tbl_len: cmd.gid_tbl_len,
	          core_cap_flags: cmd.core_cap_flags,
	          max_mad_size: cmd.max_mad_size,
        }),
    })
}

fn parse_request_set_port_attr(req: oveyd_req_pkt) -> Result<OveydReq, io::Error> {
    let cmd: oveyd_set_port_attr = unsafe {
        req.cmd.set_port_attr
    };

    Ok(OveydReq{
        seq: req.seq,
        network: Uuid::from_bytes(req.network),
        device: Some(Uuid::from_bytes(req.device)),
        port: Some(req.port),
        query: Box::new(SetPortAttrQuery{
            lid: cmd.lid,
        }),
    })
}

fn parse_request(buffer: Vec<u8>) -> Result<OveydReq, io::Error> {
    if buffer.len() < size_of::<oveyd_req_pkt>() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Too short"));
    }
    let pkt: oveyd_req_pkt = unsafe {
        let pkt = std::ptr::read(buffer.as_ptr() as *const _);
        pkt
    };

    println!("Read {}: {:?} hdr", buffer.len(), buffer);

    println!("PKT: cmd {} len {} seq {}", pkt.cmd_type, pkt.len, pkt.seq);
    match pkt.cmd_type.try_into() {
        Ok(OveydRequestType::LeaseDevice) => parse_request_lease_device(pkt),
        Ok(OveydRequestType::LeaseGid) => parse_request_lease_gid(pkt),
        Ok(OveydRequestType::ResolveGid) => parse_request_resolve_gid(pkt),
        Ok(OveydRequestType::SetGid) => parse_request_set_gid(pkt),
        Ok(OveydRequestType::CreatePort) => parse_request_create_port(pkt),
        Ok(OveydRequestType::SetPortAttr) => parse_request_set_port_attr(pkt),
        Err(_) => Err(io::Error::new(io::ErrorKind::InvalidInput, "UnknownType")),
    }
}

/// Safe to use with any wholly initialized memory `ptr`
fn raw_byte_repr<'a, T>(ptr: &'a T) -> &'a [u8]
{
    let p: *const T = ptr;
    let p: *const u8 = p as *const u8;
    let s: &[u8] = unsafe {
        slice::from_raw_parts(p, mem::size_of::<T>())
    };
    s
}

fn reply_request(file: &mut std::fs::File, resp: OveydResp) {
    let c_resp = match resp.cmd {
        OveydCmdResp::LeaseDevice(cmd) => {
            oveyd_resp_pkt{
                cmd_type: OveydRequestType::LeaseDevice as u16,
                len: size_of::<oveyd_resp_pkt>() as u16,
                seq: resp.seq,
                cmd: cmd_union{
                    lease_device: oveyd_lease_device{
                        guid: cmd.guid.into(),
                    },
                },
            }
        },
        OveydCmdResp::LeaseGid(cmd) => {
            oveyd_resp_pkt{
                cmd_type: OveydRequestType::LeaseGid as u16,
                len: size_of::<oveyd_resp_pkt>() as u16,
                seq: resp.seq,
                cmd: cmd_union{
                    lease_gid: oveyd_lease_gid{
                        idx: cmd.idx,
                        subnet_prefix: cmd.subnet_prefix.into(),
                        interface_id: cmd.interface_id.into(),
                    },
                },
            }
        },
        OveydCmdResp::ResolveGid(cmd) => {
            oveyd_resp_pkt{
                cmd_type: OveydRequestType::ResolveGid as u16,
                len: size_of::<oveyd_resp_pkt>() as u16,
                seq: resp.seq,
                cmd: cmd_union{
                    resolve_gid: oveyd_resolve_gid{
                        subnet_prefix: cmd.subnet_prefix.into(),
                        interface_id: cmd.interface_id.into(),
                    },
                },
            }
        },
        OveydCmdResp::SetGid(cmd) => {
            oveyd_resp_pkt{
                cmd_type: OveydRequestType::SetGid as u16,
                len: size_of::<oveyd_resp_pkt>() as u16,
                seq: resp.seq,
                cmd: cmd_union{
                    set_gid: oveyd_set_gid{
                        real_idx: cmd.real_idx,
                        virt_idx: cmd.virt_idx,
                        virt_subnet_prefix: cmd.virt_subnet_prefix.into(),
                        virt_interface_id: cmd.virt_interface_id.into(),
                        real_subnet_prefix: cmd.real_subnet_prefix.into(),
                        real_interface_id: cmd.real_interface_id.into(),
                    },
                },
            }
        },
        OveydCmdResp::CreatePort(cmd) => {
            oveyd_resp_pkt{
                cmd_type: OveydRequestType::CreatePort as u16,
                len: size_of::<oveyd_resp_pkt>() as u16,
                seq: resp.seq,
                cmd: cmd_union{
                    create_port: oveyd_create_port{
                        port: cmd.port,
	                      pkey_tbl_len: cmd.pkey_tbl_len,
	                      gid_tbl_len: cmd.gid_tbl_len,
	                      core_cap_flags: cmd.core_cap_flags,
	                      max_mad_size: cmd.max_mad_size,
                    },
                },
            }
        },
        OveydCmdResp::SetPortAttr(cmd) => {
            oveyd_resp_pkt{
                cmd_type: OveydRequestType::SetPortAttr as u16,
                len: size_of::<oveyd_resp_pkt>() as u16,
                seq: resp.seq,
                cmd: cmd_union{
                    set_port_attr: oveyd_set_port_attr{
                        lid: cmd.lid,
                    },
                },
            }
        },
    };
    let buf = raw_byte_repr(&c_resp);
    let _ = file.write(buf).unwrap();
}

pub async fn process_request(req: OveydReq, host: String) -> Result<OveydResp, io::Error> {
    let client = Client::new();
    let res = client
        .request(req.query.method(),
                 req.query.compile(Some(&host), req.network, req.device, req.port))
        .send()
        .await
        .unwrap();

    let cmd = match res.status() {
        StatusCode::OK | StatusCode::CREATED => {
            req.query.parse_response(res.text().await
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?)?
        }
        s => {
            println!("Received response: {:#?}", s);
            return Err(io::Error::new(io::ErrorKind::InvalidData, ""));
        }
    };

    Ok(OveydResp{
        seq: req.seq,
        network: req.network,
        cmd: cmd,
    })
}

pub async fn cdev_thread(exit_work_loop: Arc<AtomicBool>) {
    info!("Kernel request listening loop started in a thread");
    let mut file = OpenOptions::new()
        .read(true).write(true)
        .open("/dev/ovey").unwrap();

    loop {
        if exit_work_loop.load(Ordering::Relaxed) {
            info!("Received signal to exit.");
            break;
        }

        let mut buffer: Vec<u8> = vec![0; 128 as usize];
        let res = file.read(&mut buffer);
        match res {
            Err(err) if err.kind() == io::ErrorKind::Interrupted =>
                continue,
            Ok(size) if size == 0 => {
                thread::sleep(time::Duration::from_millis(500));
                continue;
            },
            Err(err) => {
                panic!("Failed read with: {}", err);
            },
            Ok(_) => {
                // Read something let's check it out
            }
        }

        let req = parse_request(buffer).unwrap();
        println!("Request parsed: {:#?}", req);
        let host = CONFIG.get_coordinator(&req.network)
            .expect("Coordinator not found for the network");
        let resp = process_request(req, host).await.unwrap();
        println!("Response: {:#?}", resp);
        reply_request(&mut file, resp);
    }
    info!("Kernel request listening loop thread done. Consider restarting Ovey daemon.");
}

#[tokio::main]
pub async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    debug!("Ovey daemon started with the following initial configuration:");
    debug!("{:#?}", *CONFIG);

    let exit_loop = Arc::new(AtomicBool::new(false));
    cdev_thread(exit_loop.clone()).await;

    // Important that this value lives through the whole lifetime of main()
    on_shutdown_move!({
        // wait for thread to finish
        exit_loop.store(true, Ordering::Relaxed);
        debug!("thread finished");
    });

    Ok(())
}
