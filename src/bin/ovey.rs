use rdma_ovey::genlink::{GenlinkAdapter, ControlAttr};
use clap::{App, Arg, SubCommand, ArgMatches};

const FAMILY_NAME: &str = "rdma-ovey";

fn main() {
    let mut verbosity = 0;

    // TODO this stucks if rdma-ovey doesn't exist? it should fail..
    let ga = GenlinkAdapter::connect(FAMILY_NAME);
    println!("Family id of {} is {}", FAMILY_NAME, ga.family_id());

    // if args are invalid this function will exit the program
    let matches = assert_and_get_args();
    verbosity = matches.occurrences_of("v");
    // println!("{:#?}", matches);


    if let Some(matches) = matches.subcommand_matches("new") {
        create_new_device(verbosity, matches, ga);
    } else if let Some(matches) = matches.subcommand_matches("delete") {
        delete_device(verbosity, matches, ga);
    }

}

fn create_new_device(verbosity: u64, matches: &ArgMatches, ga: GenlinkAdapter) {
    let matches = matches.subcommand_matches("new").unwrap();
    let new_device_name = matches.value_of("name").unwrap(); // unwrap because required
    let parent_device_name = matches.value_of("parent");
    if verbosity > 0 {
        println!("create new device: name={}, parent={}", new_device_name, parent_device_name.unwrap_or(""));
    }
}

fn delete_device(verbosity: u64, matches: &ArgMatches, ga: GenlinkAdapter) {

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
        .get_matches()
}
