use clap::ArgMatches;
use ovey_cli::cli::assert_and_get_args;
use ovey_daemon::structs::{CreateDeviceInput, CreateDeviceInputBuilder, DeleteDeviceInput, DeleteDeviceInputBuilder};
use uuid::Uuid;
use crate::daemon::{forward_create_to_daemon, forward_delete_to_daemon};

mod daemon;

fn main() {
    // if args are invalid this function will exit the program
    let matches = assert_and_get_args();
    let verbosity = matches.occurrences_of("v") as u8;
    // println!("{:#?}", matches);

   /* let ga = Ocp::connect(FAMILY_NAME, verbosity).unwrap();
    println!("Family id of {} is {}", FAMILY_NAME, ga.family_id());*/

    if let Some(matches) = matches.subcommand_matches("new") {
        action_create_new_device(verbosity, matches);
    } else if let Some(matches) = matches.subcommand_matches("delete") {
        action_delete_device(verbosity, matches);
    } else if let Some(matches) = matches.subcommand_matches("echo") {
        action_echo(verbosity, matches);
    } else {
        eprintln!("Usage: ovey -h");
    }

}

fn action_create_new_device(verbosity: u8, matches: &ArgMatches) {
    let new_device_name = matches.value_of("name").unwrap(); // unwrap because required
    let parent_device_name = matches.value_of("parent").unwrap();
    let guid_str = matches.value_of("guid").unwrap();
    let network_uuid_str = matches.value_of("vnetid").unwrap();
    let network_uuid = Uuid::parse_str(network_uuid_str).unwrap();

    if verbosity > 0 {
        println!("sending request to create new device: name={}, parent={} in network={}", new_device_name, parent_device_name, network_uuid);
    }

    // build request body for REST request to Ovey daemon
    let input: Result<CreateDeviceInput, String> = CreateDeviceInputBuilder::default()
        .virt_guid(guid_str)
        .device_name(new_device_name)
        .parent_device_name(parent_device_name)
        .network_id(network_uuid)
        .build();
    match input {
        Ok(val) => {
            let res = forward_create_to_daemon(val);
            match res {
                Ok(dto) => {
                    if verbosity > 0 {
                        println!("Ovey device was created successfully. Response from daemon:");
                        println!("{:#?}", dto);
                    }
                }
                Err(err) => {
                    eprintln!("Ovey device was NOT CREATED SUCCESSFULLY. Error from Ovey daemon:");
                    eprintln!("{}", err);
                }
            }

        }
        Err(err) => {
            eprintln!("Cannot create device. Malformed input. {}", err);
        }
    }
}

fn action_delete_device(verbosity: u8, matches: &ArgMatches) {
    let device_name = matches.value_of("name").unwrap();
    let network_uuid_str = matches.value_of("vnetid").unwrap();
    let network_uuid = Uuid::parse_str(network_uuid_str).unwrap();
    if verbosity > 0 {
        println!("sending request to delete device: name={} in network={}", device_name, network_uuid_str);
    }

    // build request body for REST request to Ovey daemon
    let input: Result<DeleteDeviceInput, String> = DeleteDeviceInputBuilder::default()
        .device_name(device_name)
        .network_id(network_uuid)
        .build();
    match input {
        Ok(val) => {
            let res = forward_delete_to_daemon(val);
            match res {
                Ok(dto) => {
                    if verbosity > 0 {
                        println!("Ovey device was deleted successfully. Response from daemon:");
                        println!("{:#?}", dto);
                    }
                }
                Err(err) => {
                    eprintln!("Ovey device was NOT DELETED SUCCESSFULLY. Error from Ovey daemon:");
                    eprintln!("{}", err);
                }
            }

        }
        Err(err) => {
            eprintln!("Cannot delete device. Malformed input. {}", err);
        }
    }
}

fn action_echo(verbosity: u8, matches: &ArgMatches) {
    let value = matches.value_of("value").unwrap(); // unwrap
    if verbosity > 0 {
        println!("sending echo request with value={}", value);
    }

    // TODO!!
    //let res = forward_echo_to_daemon()
    //println!("Received from kernel: {}", res.get_msg().unwrap());*/
}

