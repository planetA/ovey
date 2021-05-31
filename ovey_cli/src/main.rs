use clap::ArgMatches;
use std::sync::Arc;
use ovey_cli::cli::assert_and_get_args;
use ovey_daemon::structs::{CreateDeviceInput, CreateDeviceInputBuilder,
                           DeleteDeviceInput, DeletionStateDto,
                           DeviceInfoDto,
                           DeletionStateDtoBuilder, DeleteDeviceInputBuilder};
use liboveyutil::guid::{guid_string_to_u64, guid_u64_to_string};
use liboveyutil::types::GuidString;
use liboveyutil::lid::lid_string_to_u16;
use liboveyutil::types::{VirtualNetworkIdType, Uuid};
use libocp::ocp_core::{Ocp, OCPRecData};
use actix_http::http::StatusCode;
use ovey_daemon::coordinator_rest::structs::{VirtualizedDeviceDTO, VirtualizedDeviceInputBuilder};
use ovey_cli::util::get_all_local_ovey_devices;
use std::str::FromStr;
use log::debug;

type MyResult = std::result::Result<(), String>;

lazy_static::lazy_static! {
    pub(crate) static ref OCP: Arc<Ocp> = {
        Arc::from(
            Ocp::connect().expect("OCP connection must work in order for Ovey daemon to work.")
        )
    };
}

// fn get_host(network_id: &VirtualNetworkIdType) -> Result<String, DaemonRestError> {
//     // http://localhost or http://123.56.78.1 or https://foo.bar
//     let host = CONFIG.coordinators().get(network_id);
//     let host = host.ok_or(DaemonRestError::UnknownNetwork(network_id.to_owned()))?;
//     let port = OVEY_COORDINATOR_PORT;

//     let url = format!("{}:{}", host, port);

//     Ok(url)
// }

fn get_host(_network_id: &VirtualNetworkIdType) -> Result<String, String> {
    Ok("nadu1:13337".to_string())
}

/// Forwards the request from the CLI to create a device to the coordinator.
/// Returns the DTO from the coordinator on success.
pub fn rest_forward_create_device(input: CreateDeviceInput, physical_guid_str: GuidString) -> Result<VirtualizedDeviceDTO, String> {
    let host = get_host(input.network_id())?;
    // endpoint inside REST service with starting /
    let endpoint = ovey_coordinator::urls::build_add_device_url(input.network_id().to_owned());
    let url = format!("http://{}{}", host, endpoint);

    // Transform payload from cli request to payload for ovey coordinator
    let payload = VirtualizedDeviceInputBuilder::default()
        .virtual_device_guid_string(input.virt_guid())
        .physical_device_guid_string(physical_guid_str)
        .parent_device_name(input.parent_device_name())
        .device_name(input.device_name())
        .build()?;

    let client = reqwest::blocking::Client::new();
    let res = client.post(&url)
        .json(&payload)
        .send()
        .map_err(|e| format!("Failed post requset at {}: {}", url, e))?;

    println!("Post {}: {:#?}", url, res);
    if res.status() == StatusCode::NOT_FOUND {
        return Err(format!("Device does not exist: {} {}",
            input.virt_guid().to_owned(),
            input.network_id().to_owned())
        );
    }
    if res.status() == StatusCode::CONFLICT {
        return Err(format!("Device already registered: {} {}",
            input.virt_guid().to_owned(),
            input.network_id().to_owned())
        );
    }

    let res = res.json::<VirtualizedDeviceDTO>().map_err(|e| format!("Failed to convert: {}", e))?;
    Ok(res)
}

fn main() {
    // if args are invalid this function will exit the program
    let matches = assert_and_get_args();
    let verbosity = matches.occurrences_of("v") as u8;
    env_logger::init();
    // println!("{:#?}", matches);

   /* let ga = Ocp::connect(FAMILY_NAME, verbosity).unwrap();
    println!("Family id of {} is {}", FAMILY_NAME, ga.family_id());*/

    let res = if let Some(matches) = matches.subcommand_matches("new") {
        action_create_new_device(verbosity, matches)
    } else if let Some(matches) = matches.subcommand_matches("delete") {
        action_delete_device(verbosity, matches)
    }  else if let Some(matches) = matches.subcommand_matches("list") {
        action_list(verbosity, matches)
    } else {
        eprintln!("Usage: ovey -h");
        Err("Unknown command".to_string())
    };

    if let Err(err) = res {
        eprintln!("Failed with: {}", err);
        std::process::exit(-1);
    }
}

fn route_post_create_device(input: CreateDeviceInput) -> Result<VirtualizedDeviceDTO, String> {
    // REGISTER DEVICE LOCALLY VIA OCP INSIDE KERNEL
    // now we first create the device on the machine
    // and then we tell the coordinator about it

    let guid = guid_string_to_u64(input.virt_guid());
    let lid = lid_string_to_u16(input.virt_lid());
    OCP.ocp_create_device(
        input.device_name(),
        input.parent_device_name(),
        guid,
        lid,
        &input.network_id().to_string(),
    ).map_err(|e| format!("failed to create device: {}", e))?;

    // check that the device was really created
    let ocp_res = OCP.ocp_get_device_info(input.device_name())
        .map_err(|e| format!("Failed to get device info: {}", e))?;



    // THIRD STEP: NOW REGISTER THE DEVICE AT COORDINATOR
    let device_name = input.device_name().to_owned(); // fix use after move with input.device_name() later needed
    let resp = rest_forward_create_device(
        input,
        guid_u64_to_string(
            ocp_res
                .parent_node_guid()
                .expect("Must exist at this point"),
        ),
    );

    // if something failed; delete device on local machine again
    if resp.is_err() {
        eprintln!("A failure occurred: {:#?}", resp.as_ref().unwrap_err());
        // OCP.ocp_delete_device(&device_name)
        //     .map_err(|e| format!("Failed to delete device: {}", e))?;
    }

    debug!("registering device {} at coordinator successful", device_name);

    let dto = resp?;

    Ok(dto)
}

fn action_create_new_device(verbosity: u8, matches: &ArgMatches) -> MyResult {
    let new_device_name = matches.value_of("name").unwrap(); // unwrap because required
    let parent_device_name = matches.value_of("parent").unwrap();
    let guid_str = matches.value_of("guid").unwrap();
    let lid_str = matches.value_of("lid").or(Some("42")).unwrap();
    let network_uuid_str = matches.value_of("vnetid").unwrap();
    let network_uuid = Uuid::parse_str(network_uuid_str).unwrap();

    if verbosity > 0 {
        println!("sending request to create new device: name={}, parent={} in network={}", new_device_name, parent_device_name, network_uuid);
    }

    // build request body for REST request to Ovey daemon
    let input: Result<CreateDeviceInput, String> = CreateDeviceInputBuilder::default()
        .virt_guid(guid_str)
        .virt_lid(lid_str)
        .device_name(new_device_name)
        .parent_device_name(parent_device_name)
        .network_id(network_uuid)
        .build();
    match input {
        Ok(val) => {
            println!("Creating device: {:#?}", val);
            let res = route_post_create_device(val);
            match res {
                Ok(dto) => {
                    if verbosity > 0 {
                        println!("Ovey device was created successfully. Response from daemon:");
                        println!("{:#?}", dto);
                    }
                    Ok(())
                }
                Err(err) => {
                    eprintln!("Ovey device was NOT CREATED SUCCESSFULLY. Error from Ovey daemon:");
                    eprintln!("{}", err);
                    Err(err)
                }
            }

        }
        Err(err) => {
            eprintln!("Cannot create device. Malformed input. {}", err);
            Err(err)
        }
    }
}

/// Forwards the request from the CLI to delete a device to the coordinator.
/// Returns the DTO from the coordinator on success.
pub fn rest_forward_delete_device(device_id: &GuidString, network_id: &VirtualNetworkIdType) -> Result<VirtualizedDeviceDTO, String> {
    // http://localhost or http://123.56.78.1 or https://foo.bar
    let host = get_host(network_id)?;
    // endpoint inside REST service with starting /
    let endpoint = ovey_coordinator::urls::build_device_url(network_id.to_owned(), device_id.to_owned());
    let url = format!("http://{}{}", host, endpoint);

    let client = reqwest::blocking::Client::new();
    let res = client.delete(&url).send();
    let res = res.map_err(|e| format!("Failed to talk to coordinator: {}", e))?;

    if res.status() == StatusCode::NOT_FOUND {
        return Err(format!("Device does not exist: {} {}",
            device_id.to_owned(),
            network_id.to_owned())
        );
    }

    let res = res.json::<VirtualizedDeviceDTO>()
        .map_err(|e| format!("Failed to parse deletion: {}", e))?;
    Ok(res)
}

pub fn route_delete_delete_device(input: DeleteDeviceInput) -> Result<DeletionStateDto, String> {
    // verify input

    // first step; check via OCP if device is registered on local machine
    let ocp_data = OCP
        .ocp_get_device_info(input.device_name())
        .map_err(|err| format!("Failed to get device info {}", err))?;

    // second step: delete it on coordinator
    // fetch network id; we need it for the deletion request on the coordinator
    let network_id = ocp_data.virt_network_uuid_str().unwrap();
    let network_id = Uuid::parse_str(network_id)
        .map_err(|err| format!("Failed to convert to UUID: {}", err))?;
    let guid_str = guid_u64_to_string(ocp_data.node_guid().unwrap());

    // delete in both places without early canceling (no .unwrap() or ?)

    let coordinator_result = rest_forward_delete_device(&guid_str, &network_id);
    debug!("Delete at coordinator: {:#?}", coordinator_result);

    // actually delete device on local machine inside Ovey kernel module
    let ocp_result = OCP.ocp_delete_device(input.device_name())
        .map_err(|err| format!("Failed to delete device: {}", err));

    debug!("Finished delete: {:#?}", ocp_result);

    let deletion_state: DeletionStateDto = DeletionStateDtoBuilder::default()
        .deletion_local_successfully(ocp_result.is_ok())
        .deletion_local_info_msg(
            ocp_result
                .err()
                // display is implemented by a derive macro
                // even if IDE doesn't recognize it
                .map(|e| format!("{}", e)),
        )
        .deletion_coordinator_successfully(coordinator_result.is_ok())
        .deletion_coordinator_info_msg(
            coordinator_result
                .err()
                // display is implemented by a derive macro
                // even if IDE doesn't recognize it
                .map(|e| format!("{}", e)),
        )
        .build()
        .unwrap();

    Ok(deletion_state)
}

fn action_delete_device(verbosity: u8, matches: &ArgMatches) -> MyResult {
    let device_name = matches.value_of("name").unwrap();
    if verbosity > 0 {
        println!("sending request to delete device on local machine: name={}", device_name);
    }

    // build request body for REST request to Ovey daemon
    let input: Result<DeleteDeviceInput, String> = DeleteDeviceInputBuilder::default()
        .device_name(device_name)
        .build();
    match input {
        Ok(val) => {
            let res = route_delete_delete_device(val);
            match res {
                Ok(dto) => {
                    if verbosity > 0 {
                        println!("Ovey device was deleted successfully. Response from daemon:");
                        println!("{:#?}", dto);
                    }
                    Ok(())
                }
                Err(err) => {
                    eprintln!("Ovey device was NOT DELETED SUCCESSFULLY. Error from Ovey daemon:");
                    eprintln!("{}", err);
                    Err(err)
                }
            }

        }
        Err(err) => {
            eprintln!("Cannot delete device. Malformed input. {}", err);
            Err(err)
        }
    }
}

pub fn route_get_list_devices() -> Result<Vec<DeviceInfoDto>, String> {
    let devs = get_all_local_ovey_devices();
    debug!("Available local ovey devices: {:#?}", &devs);

    // the ? operator inside map seems not to work :/
    let devs = devs
        .into_iter()
        .map(|dev| {
            OCP.ocp_get_device_info(&dev)
                .map_err(|e| format!(
                        "Device not found. Could not fetch info for device '{}' via OCP. err='{}'",
                        dev, e
                    ),
                )
        })
        .collect::<Vec<Result<OCPRecData, String>>>();

    // check if there is any error and unwrap the first one
    let errs = devs
        .iter()
        .filter(|x| x.is_err())
        .map(|x| x.as_ref().unwrap_err())
        .collect::<Vec<&String>>();
    if !errs.is_empty() {
        // return error
        return Err(errs[0].clone());
    }

    // now that we know there are no errors unwrap all real objects
    let devs = devs
        .into_iter()
        .map(|x| x.unwrap())
        .collect::<Vec<OCPRecData>>();

    let devs = devs
        .into_iter()
        .map(|data| {
            DeviceInfoDto{
                dev_name: data.device_name().unwrap().to_string(),
                parent_dev_name: data.parent_device_name().unwrap().to_string(),
                guid: data.node_guid().unwrap(),
                lid: data.node_lid().or(Some(42)).unwrap(),
                parent_guid: data.parent_node_guid().unwrap(),
                virtual_network_id: Uuid::from_str(data.virt_network_uuid_str().unwrap()).unwrap()
            }
        })
        .collect::<Vec<DeviceInfoDto>>();

    Ok(devs)
}

/// Queries the daemon and returns information about all local Ovey devices.
fn action_list(_verbosity: u8, _matches: &ArgMatches) -> MyResult {
    let res = route_get_list_devices();
    match res {
        Ok(data) => {
            println!("Found the following Ovey devices:");
            // TODO make this pretty
            println!("{:#?}", data);
            Ok(())
        }
        Err(e) => {
            eprintln!("Cannot list devices. Couldn't fetch data from Ovey daemon ({}).", e);
            Err(e)
        }
    }
}
