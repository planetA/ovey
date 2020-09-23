use rdma_ovey::ocp::{Ocp, OveyOperation, build_nl_attrs, OveyAttribute};
use clap::ArgMatches;
use rdma_ovey::cli::assert_and_get_args;
use rdma_ovey::ocp::ocp_properties::{FAMILY_NAME, OveyOperation, OveyAttribute};
use rdma_ovey::ocp::ocp_core::{Ocp, build_nl_attrs};

fn main() {
    // if args are invalid this function will exit the program
    let matches = assert_and_get_args();
    let verbosity = matches.occurrences_of("v") as u8;
    // println!("{:#?}", matches);

    let ga = Ocp::connect(FAMILY_NAME, verbosity).unwrap();
    println!("Family id of {} is {}", FAMILY_NAME, ga.family_id());


    if let Some(matches) = matches.subcommand_matches("new") {
        nl_create_new_device(verbosity, matches, ga);
    } else if let Some(matches) = matches.subcommand_matches("delete") {
        nl_delete_device(verbosity, matches, ga);
    } else if let Some(matches) = matches.subcommand_matches("echo") {
        nl_echo(verbosity, matches, ga);
    } else {
        panic!("Provide at least one subcommand!");
    }

}

fn nl_create_new_device(verbosity: u8, matches: &ArgMatches, mut ga: Ocp) {
    let new_device_name = matches.value_of("name").unwrap(); // unwrap because required
    let parent_device_name = matches.value_of("parent").unwrap();
    if verbosity > 0 {
        println!("sending request to create new device: name={}, parent={}", new_device_name, parent_device_name);
    }
    let _res = ga.send_and_ack(
        OveyOperation::CreateDevice,
        build_nl_attrs(
            vec![
                (OveyAttribute::DeviceName, new_device_name),
                (OveyAttribute::ParentDeviceName, parent_device_name),
            ]
        )
    ).unwrap();
}

fn nl_delete_device(verbosity: u8, matches: &ArgMatches, mut ga: Ocp) {
    let device_name = matches.value_of("name").unwrap(); // unwrap
    if verbosity > 0 {
        println!("sending request to delete device: name={}", device_name);
    }
    let _res = ga.send_single_and_ack(
        OveyOperation::DeleteDevice,
        OveyAttribute::DeviceName,
        device_name
    ).unwrap();
}

fn nl_echo(verbosity: u8, matches: &ArgMatches, mut ga: Ocp) {
    let value = matches.value_of("value").unwrap(); // unwrap
    if verbosity > 0 {
        println!("sending echo request for value={}", value);
    }
    let res = ga.send_single_and_ack(
        OveyOperation::Echo,
        OveyAttribute::Msg,
        value
    ).unwrap();

    println!("Received from kernel: {}", res.get_msg().unwrap());
}

