//! Ovey CLI talks with and only with Ovey daemon.
//! This module is the glue between CLI and daemon.

use ovey_daemon::cli_rest_api::CreateDeviceInput;
use ovey_daemon::rest::structs::VirtualizedDeviceDTO;

pub fn forward_create_to_daemon(input: CreateDeviceInput) -> Result<VirtualizedDeviceDTO, String> {
    let req = reqwest::blocking::Client::new();
    let res = req.post(

    )
}
