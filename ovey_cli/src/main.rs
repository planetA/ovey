use clap::ArgMatches;
use libocp::ocp_properties::{FAMILY_NAME, OveyOperation, OveyAttribute};
use libocp::ocp_core::{Ocp, build_nl_attr};
use librdmautil::guid_string_to_ube64;
use ovey_cli::cli::assert_and_get_args;
use ovey_daemon::structs::{CreateDeviceInput, CreateDeviceInputBuilder};
use uuid::Uuid;
use crate::daemon::forward_create_to_daemon;

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

    if verbosity > 0 {
        println!("sending request to create new device: name={}, parent={}", new_device_name, parent_device_name);
    }

    // build request body for REST request to Ovey daemon
    let input: Result<CreateDeviceInput, String> = CreateDeviceInputBuilder::default()
        .virt_guid(guid_str)
        .device_name(new_device_name)
        .parent_device_name(parent_device_name)
        .network_id(Uuid::parse_str(network_uuid_str).unwrap())
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

    /*// "real" big endian
    let guid_be = guid_string_to_ube64(guid_str);
    // host endianness
    let guid_he = u64::from_be(guid_be);

    let _res = ga.send_and_ack(
        OveyOperation::CreateDevice,
        vec![
            build_nl_attr(OveyAttribute::DeviceName, new_device_name),
            build_nl_attr(OveyAttribute::ParentDeviceName, parent_device_name),
            build_nl_attr(OveyAttribute::NodeGuid, guid_he),
        ]
        // doesn't work due to conflicting generic type in build_nl_attrs
        /*build_nl_attrs(
            vec![
                (OveyAttribute::DeviceName, new_device_name),
                (OveyAttribute::ParentDeviceName, parent_device_name),
                (OveyAttribute::NodeGuid, guid),
            ]
        )*/
    ).unwrap();*/
}

fn action_delete_device(verbosity: u8, matches: &ArgMatches) {
    let device_name = matches.value_of("name").unwrap(); // unwrap
    if verbosity > 0 {
        println!("sending request to delete device: name={}", device_name);
    }
    /*let _res = ga.send_single_and_ack(
        OveyOperation::DeleteDevice,
        OveyAttribute::DeviceName,
        device_name
    ).unwrap();*/
}

fn action_echo(verbosity: u8, matches: &ArgMatches) {
    let value = matches.value_of("value").unwrap(); // unwrap
    if verbosity > 0 {
        println!("sending echo request with value={}", value);
    }
    /*let res = ga.send_single_and_ack(
        OveyOperation::Echo,
        OveyAttribute::Msg,
        value
    ).unwrap();

    println!("Received from kernel: {}", res.get_msg().unwrap());*/
}

