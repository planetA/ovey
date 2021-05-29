//! Ovey CLI talks with and only with Ovey daemon.
//! This module is the glue between CLI and daemon via REST.

use ovey_daemon::structs::{DeleteDeviceInput, DeletionStateDto, DeviceInfoDto};
use ovey_daemon::consts::OVEY_DAEMON_PORT;
use reqwest::StatusCode;

pub fn forward_list_to_daemon() -> Result<Vec<DeviceInfoDto>, String> {
    let req = reqwest::blocking::Client::new();
    let host = format!("http://localhost:{}", OVEY_DAEMON_PORT);
    let endpoint = ovey_daemon::urls::ROUTE_DEVICES;
    let url = format!("{}{}", host, endpoint);
    let res = req.get(&url).send();
    let res = res.map_err(|e| format!("Daemon didn't responded successfully: {}", e.to_string()))?;
    match res.status() {
        StatusCode::OK => {
            let dto = res.json::<Vec<DeviceInfoDto>>()
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

// TODO
/*pub fn forward_echo_to_daemon(_msg: String) -> Result<String, String> {
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
}*/
