use std::io::Write;
use std::io::Read;
use config::CONFIG;
use std::{thread, time};
use std::sync::Arc;
use std::fs::{File, OpenOptions};
use simple_on_shutdown::on_shutdown_move;
use std::sync::atomic::{AtomicBool, Ordering};
use futures::executor::block_on;
use std::thread::{JoinHandle, spawn};
use std::mem::size_of;
use std::io;
use serde::{Deserialize, Serialize};
use std::convert::TryInto;
use std::convert::TryFrom;
use std::slice;
use std::mem;
use reqwest::StatusCode;
use liboveyutil::types::*;

mod config;
use uuid::Uuid;

#[macro_use]
extern crate log;

enum OveydRequestType {
    LeaseDevice = 0,
    LeaseGid = 1,
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
    port: u16,
    idx: u32,
    subnet_prefix: U64Be,
    interface_id: U64Be,
}

#[repr(C)]
union cmd_union {
    pub lease_device: oveyd_lease_device,
    pub lease_gid: oveyd_lease_gid,
}

#[repr(C)]
struct oveyd_req_pkt {
    pub cmd_type: u16,
    pub len: u16,
    pub seq: u32,
    pub network: [u8; 16],
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
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
struct OveydReq {
    seq: u32,
    network: Uuid,
    cmd: OveydCmd,
}

#[derive(Debug)]
pub enum OveydCmd {
    LeaseDevice(LeaseDeviceReq),
    LeaseGid(LeaseGidReq)
}

#[derive(Debug)]
pub enum OveydCmdResp {
    LeaseDevice(LeaseDeviceResp),
    LeaseGid(LeaseGidResp)
}

pub struct OveydResp {
    seq: u32,
    network: Uuid,
    cmd: OveydCmdResp,
}

fn parse_request_lease_device(req: oveyd_req_pkt) -> Result<OveydReq, io::Error> {
    let cmd: oveyd_lease_device = unsafe {
        req.cmd.lease_device
    };

    Ok(OveydReq{
        seq: req.seq,
        network: Uuid::from_bytes(req.network),
        cmd: OveydCmd::LeaseDevice(LeaseDeviceReq{
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
        cmd: OveydCmd::LeaseGid(LeaseGidReq{
            port: cmd.port,
            idx: cmd.idx,
            subnet_prefix: u64::from_be(cmd.subnet_prefix.0),
            interface_id: u64::from_be(cmd.interface_id.0),
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

    match pkt.cmd_type.try_into() {
        Ok(OveydRequestType::LeaseDevice) => parse_request_lease_device(pkt),
        Ok(OveydRequestType::LeaseGid) => parse_request_lease_gid(pkt),
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
    match resp.cmd {
        OveydCmdResp::LeaseDevice(cmd) => {
            let c_resp = oveyd_resp_pkt{
                cmd_type: OveydRequestType::LeaseDevice as u16,
                len: size_of::<oveyd_resp_pkt>() as u16,
                seq: resp.seq,
                cmd: cmd_union{
                    lease_device: oveyd_lease_device{
                        guid: cmd.guid.into(),
                    },
                },
            };
            println!("Size struct {}", size_of::<oveyd_resp_pkt>());
            let buf = raw_byte_repr(&c_resp);
            println!("Size struct {}", buf.len());
            let size = file.write(buf).unwrap();
            println!("Wrote struct size {}", size);
        },
        OveydCmdResp::LeaseGid(cmd) => {
            let c_resp = oveyd_resp_pkt{
                cmd_type: OveydRequestType::LeaseGid as u16,
                len: size_of::<oveyd_resp_pkt>() as u16,
                seq: resp.seq,
                cmd: cmd_union{
                    lease_gid: oveyd_lease_gid{
                        port: cmd.port,
                        idx: cmd.idx,
                        subnet_prefix: cmd.subnet_prefix.into(),
                        interface_id: cmd.interface_id.into(),
                    },
                },
            };
            println!("Size struct {}", size_of::<oveyd_resp_pkt>());
            let buf = raw_byte_repr(&c_resp);
            println!("Size struct {}", buf.len());
            let size = file.write(buf).unwrap();
            println!("Wrote struct size {}", size);
        }
    }
}

fn process_request(req: OveydReq) -> Result<OveydResp, io::Error> {
    println!("Request parsed: {:#?}", req);

    let mut host = CONFIG.get_coordinator(&req.network)
        .ok_or(io::Error::new(io::ErrorKind::NotFound,
                           "Coordinator not found for the network"))?;
    let cmd = match req.cmd {
        OveydCmd::LeaseDevice(cmd) => {
            let endpoint = ovey_coordinator::urls::build_add_device_url(req.network);
            host.push_str(&endpoint);
            let client = reqwest::blocking::Client::new();
            let res = client.post(&host).json(&cmd)
                .send()
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

            match res.status() {
                StatusCode::OK => {
                    OveydCmdResp::LeaseDevice(res.json::<LeaseDeviceResp>()
                                              .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?)
                },
                s => {
                    println!("Received response: {}", s);
                    return Err(io::Error::new(io::ErrorKind::InvalidData, ""));
                }
            }
        },
        OveydCmd::LeaseGid(cmd) => {
            let endpoint = ovey_coordinator::urls::build_lease_gid_url(req.network);
            host.push_str(&endpoint);
            let client = reqwest::blocking::Client::new();
            let res = client.post(&host).json(&cmd)
                .send()
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

            match res.status() {
                StatusCode::OK => {
                    OveydCmdResp::LeaseGid(res.json::<LeaseGidResp>()
                        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?)
                },
                s => {
                    println!("Received response: {}", s);
                    return Err(io::Error::new(io::ErrorKind::InvalidData, ""));
                }
            }
        }
    };

    Ok(OveydResp{
        seq: req.seq,
        network: req.network,
        cmd: cmd,
    })
}

pub fn cdev_thread(exit_work_loop: Arc<AtomicBool>) -> JoinHandle<()> {
    spawn(move || {
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

            let req = parse_request(buffer);
            match req.and_then(|req| process_request(req)) {
                Ok(resp) => {
                    reply_request(&mut file, resp);
                },
                Err(err) => {
                    println!("Failed to parse request: {}", err);
                }
            }

        }
        info!("Kernel request listening loop thread done. Consider restarting Ovey daemon.");
    })
}

fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    debug!("Ovey daemon started with the following initial configuration:");
    debug!("{:#?}", *CONFIG);

    let exit_loop = Arc::new(AtomicBool::new(false));
    let loop_thread_handle = cdev_thread(exit_loop.clone());

    // Important that this value lives through the whole lifetime of main()
    on_shutdown_move!({
        // wait for thread to finish
        exit_loop.store(true, Ordering::Relaxed);
        debug!("thread finished");
    });

    // check if all coordinators are up and valid
    loop_thread_handle.join().unwrap();

    Ok(())
}
