use std::io::Read;
use config::CONFIG;
use std::{thread, time};
use std::sync::Arc;
use std::fs::File;
use simple_on_shutdown::on_shutdown_move;
use std::sync::atomic::{AtomicBool, Ordering};
use futures::executor::block_on;
use std::thread::{JoinHandle, spawn};
use std::mem::size_of;
use std::io;
use serde::{Deserialize, Serialize};
use std::convert::TryInto;
use std::convert::TryFrom;
use liboveyutil::types::*;

mod config;
use uuid::Uuid;

#[macro_use]
extern crate log;

enum OveydRequestType {
    LeaseDevice = 0,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct oveydr_req_hdr {
    pub req_type: u16,
    pub len: u16,
    pub seq: u32,
    pub network: [u8; 16],
}

// Big endian u64 type
type U64Be = u64;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct oveydr_lease_device {
    pub hdr: oveydr_req_hdr,
    guid: U64Be,
}

impl TryFrom<u16> for OveydRequestType {
    type Error = ();

    fn try_from(v: u16) -> Result<Self, Self::Error> {
        match v {
            x if x == OveydRequestType::LeaseDevice as u16 => Ok(OveydRequestType::LeaseDevice),
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
    LeaseDevice(LeaseDeviceReq)
}

fn parse_request_lease_device(buffer: Vec<u8>) -> Result<OveydReq, io::Error> {
    if buffer.len() < size_of::<oveydr_lease_device>() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Too short"));
    }
    let req: oveydr_lease_device = unsafe {
        std::ptr::read(buffer.as_ptr() as *const _)
    };

    Ok(OveydReq{
        seq: req.hdr.seq,
        network: Uuid::from_bytes(req.hdr.network),
        cmd: OveydCmd::LeaseDevice(LeaseDeviceReq{
            guid: u64::from_be(req.guid),
        }),
    })
}

fn parse_request(buffer: Vec<u8>) -> Result<OveydReq, io::Error> {
    if buffer.len() < size_of::<oveydr_req_hdr>() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Too short"));
    }
    let hdr: oveydr_req_hdr = unsafe {
        std::ptr::read(buffer.as_ptr() as *const _)
    };

    println!("Read {}: {:?} hdr {:#?}", buffer.len(), buffer, hdr);

    match hdr.req_type.try_into() {
        Ok(OveydRequestType::LeaseDevice) => parse_request_lease_device(buffer),
        Err(_) => Err(io::Error::new(io::ErrorKind::InvalidInput, "UnknownType")),
    }
}

fn process_request(req: OveydReq) -> Result<(), io::Error> {
    let mut host = CONFIG.get_coordinator(&req.network)
        .ok_or(io::Error::new(io::ErrorKind::NotFound,
                           "Coordinator not found for the network"))?;
    match req.cmd {
        OveydCmd::LeaseDevice(cmd) => {
            let endpoint = ovey_coordinator::urls::build_add_device_url(req.network);
            host.push_str(&endpoint);
            let client = reqwest::blocking::Client::new();
            let res = client.post(&host).json(&cmd).send();

            println!("Received reply: {:#?}", res);
        }
    }
    Ok(())
}

pub fn cdev_thread(exit_work_loop: Arc<AtomicBool>) -> JoinHandle<()> {
    spawn(move || {
        info!("Kernel request listening loop started in a thread");
        let mut file = File::open("/dev/ovey").unwrap();

        loop {
            if exit_work_loop.load(Ordering::Relaxed) {
                info!("Received signal to exit.");
                break;
            }

            let mut buffer: Vec<u8> = vec![0; 128 as usize];
            let size = file.read(&mut buffer).unwrap();
            if size == 0 {
                thread::sleep(time::Duration::from_millis(500));
                continue;
            }

            let req = parse_request(buffer);
            match req {
                Ok(req) => {
                    println!("Request parsed: {:#?}", req);
                    if let Err(err) = process_request(req) {
                        println!("Failed to process request: {}", err);
                    }
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
