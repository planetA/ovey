use rdma_ovey::genlink::{GenlinkAdapter, OveyOperation, build_nl_attrs, OveyAttribute, build_nl_attr};
use clap::{App, Arg, SubCommand, ArgMatches};

const FAMILY_NAME: &str = "rdma-ovey";

fn main() {
    let mut verbosity;

    // TODO this stucks if rdma-ovey doesn't exist? it should fail..
    let ga = GenlinkAdapter::connect(FAMILY_NAME);
    println!("Family id of {} is {}", FAMILY_NAME, ga.family_id());

    // if args are invalid this function will exit the program
    let matches = assert_and_get_args();
    verbosity = matches.occurrences_of("v");
    // println!("{:#?}", matches);


    if let Some(matches) = matches.subcommand_matches("new") {
        nl_create_new_device(verbosity, matches, ga);
    } else if let Some(matches) = matches.subcommand_matches("delete") {
        nl_delete_device(verbosity, matches, ga);
    } else if let Some(matches) = matches.subcommand_matches("echo") {
        nl_echo(verbosity, matches, ga);
    }

}

fn nl_create_new_device(verbosity: u64, matches: &ArgMatches, mut ga: GenlinkAdapter) {
    let new_device_name = matches.value_of("name").unwrap(); // unwrap because required
    let parent_device_name = matches.value_of("parent").unwrap();
    if verbosity > 0 {
        println!("sending request to create new device: name={}, parent={}", new_device_name, parent_device_name);
    }
    ga.send(
        OveyOperation::CreateDevice,
        build_nl_attrs(
            vec![
                (OveyAttribute::DeviceName, new_device_name),
                (OveyAttribute::ParentDeviceName, parent_device_name),
            ]
        )
    );
    ga.recv_ack(OveyOperation::CreateDevice);
}

fn nl_delete_device(verbosity: u64, matches: &ArgMatches, mut ga: GenlinkAdapter) {
    let device_name = matches.value_of("name").unwrap(); // unwrap
    if verbosity > 0 {
        println!("sending request to delete device: name={}", device_name);
    }
    ga.send_single(
        OveyOperation::DeleteDevice,
        build_nl_attr(OveyAttribute::DeviceName, device_name)
    );
    // ga.recv_ack(OveyOperation::DeleteDevice);
}

fn nl_echo(verbosity: u64, matches: &ArgMatches, mut ga: GenlinkAdapter) {
    let value = matches.value_of("value").unwrap(); // unwrap
    if verbosity > 0 {
        println!("sending echo request for value={}", value);
    }
    ga.send_single(
        OveyOperation::Echo,
        build_nl_attr(OveyAttribute::Msg, value)
    );
    // ga.recv_ack();
    let msg = ga.recv_first_of_type_raw(OveyAttribute::Msg)
        .map(|bytes| String::from_utf8(bytes).unwrap())
        .unwrap();
    println!("Received from kernel: {}", msg);
}

/// Parses the args and asserts that required args are in the proper order and format.
fn assert_and_get_args<'a>() -> ArgMatches<'a> {
    App::new("Overlay RDMA network util")
        .version("1.0")
        .author("Philipp Schuster <philipp_johannes.schuster@tu-dresden.de>")
        .about(
            "Userland part of the 'ovey' project that creates virtual overlay rdma network devices"
        )
        .arg(Arg::with_name("v")
            .short("v")
            .multiple(true)
            .help("Sets the level of verbosity"))
        .subcommand(SubCommand::with_name("new")
            .display_order(0)
            .about("create virtual overlay rdma network device")
            .arg(Arg::with_name("name")
                .long("name")
                .short("n")
                .takes_value(true)
                .required(true)
                .help("device name"))
            .arg(Arg::with_name("parent")
                .long("parent")
                .short("p")
                .takes_value(true)
                .required(true)
                .help("parent device name")))
        .subcommand(SubCommand::with_name("delete")
            .display_order(1)
            .about("remove virtual overlay rdma network device")
            .arg(Arg::with_name("name")
                .long("name")
                .short("n")
                .takes_value(true)
                .required(true)
                .help("device name")))
        .subcommand(SubCommand::with_name("echo")
            .display_order(1)
            .about("sends a message via netlink and receives a message back")
            .arg(Arg::with_name("value")
                .long("value")
                .takes_value(true)
                .required(true)
                .help("text to send to kernel")))
        .get_matches()
}
