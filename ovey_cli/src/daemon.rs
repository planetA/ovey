//! Ovey CLI talks with and only with Ovey daemon.
//! This module is the glue between CLI and daemon via REST.

use ovey_daemon::structs::{CreateDeviceInput, DeleteDeviceInput, DeletionStateDto};
use ovey_daemon::coordinator_rest::structs::VirtualizedDeviceDTO;
use ovey_daemon::consts::OVEY_DAEMON_PORT;
use reqwest::StatusCode;

pub fn forward_create_to_daemon(input: CreateDeviceInput) -> Result<VirtualizedDeviceDTO, String> {
    let req = reqwest::blocking::Client::new();
    let host = format!("http://localhost:{}", OVEY_DAEMON_PORT);
    let endpoint = ovey_daemon::urls::ROUTE_DEVICE;
    let url = format!("{}{}", host, endpoint);
    let res = req.post(&url)
        .json(&input)
        .send();
    let res = res.map_err(|e| format!("Daemon didn't responded successfully: {}", e.to_string()))?;
    match res.status() {
        StatusCode::OK => {
            let dto = res.json::<VirtualizedDeviceDTO>()
                .map_err(|e| format!("Daemon sent wrong reply: {}", e.to_string()));
            dto
        },
        StatusCode::CONFLICT => {
            Err(format!("Ovey daemon said that the device is already registered! error='{}'",
                        res.text().unwrap_or("<unknown>".to_string())))
        }
        _ => {
            Err(format!("Ovey daemon responded with status code {} and error: {}",
                        res.status(),
                        res.text().unwrap_or("<unknown>".to_string())))
        }
    }
}

pub fn forward_delete_to_daemon(input: DeleteDeviceInput) -> Result<DeletionStateDto, String> {
    let req = reqwest::blocking::Client::new();
    let host = format!("http://localhost:{}", OVEY_DAEMON_PORT);
    let endpoint = ovey_daemon::urls::ROUTE_DEVICE;
    let url = format!("{}{}", host, endpoint);
    let res = req.delete(&url)
        .json(&input)
        .send();
    let res = res.map_err(|e| format!("Daemon didn't responded successfully: {}", e.to_string()))?;
    match res.status() {
        StatusCode::OK => {
            let dto = res.json::<DeletionStateDto>()
                .map_err(|e| format!("Daemon sent wrong reply: {}", e.to_string()));
            dto
        },
        _ => {
            Err(format!("Ovey daemon responded with status code {} and error: {}",
                        res.status(),
                        res.text().unwrap_or("<unknown>".to_string())))
        }
    }
}

pub fn forward_echo_to_daemon(msg: String) -> Result<String, String> {
    /*let req = reqwest::blocking::Client::new();
    let host = format!("http://localhost:{}", OVEY_DAEMON_PORT);
    let endpoint = ovey_daemon::urls::ROUTE_DEVICE;
    let url = format!("{}{}", host, endpoint);
    let res = req.delete(&url)
        .json(&input)
        .send();
    let res = res.map_err(|e| format!("Daemon didn't responded successfully: {}", e.to_string()))?;
    match res.status() {
        StatusCode::OK => {
            let dto = res.json::<VirtualizedDeviceDTO>()
                .map_err(|e| format!("Daemon sent wrong reply: {}", e.to_string()));
            dto
        },
        _ => {
            Err(format!("Ovey daemon responded with status code {} and error: {}",
                        res.status(),
                        res.text().unwrap_or("<unknown>".to_string())))
        }
    }*/
    Err("UNIMPLEMENTED!".to_owned())
}
