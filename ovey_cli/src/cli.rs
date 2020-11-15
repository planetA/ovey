//! Utilities to parse the command line

use clap::{ArgMatches, App, SubCommand, Arg};
use regex::Regex;
use libocp::ocp_properties::{DEVICE_NAME_PATTERN, PARENT_DEVICE_NAME_PATTERN};
use librdmautil::GUID_STRING_PATTERN;
use ovey_daemon::cli_rest_api::validation;

/// Parses the args and asserts that required args are in the proper order and format.
pub fn assert_and_get_args<'a>() -> ArgMatches<'a> {

    let dev_name_validator = move |name: String| {
        validation::validate_device_name(&name)
    };

    let parent_dev_name_validator = move |name: String| {
        validation::validate_parent_device_name(&name)
    };

    let guid_validator = move |name: String| {
        validation::validate_guid(&name)
    };

    App::new("Overlay RDMA network util")
        .version("1.0")
        .author("Philipp Schuster <philipp_johannes.schuster@mailbox.tu-dresden.de>")
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
                .validator(dev_name_validator)
                .help(&format!("device name (pattern: {})", DEVICE_NAME_PATTERN)))
            .arg(Arg::with_name("parent")
                .long("parent")
                .short("p")
                .takes_value(true)
                .required(true)
                .validator(parent_dev_name_validator)
                .help(&format!("parent device name (pattern: {})", PARENT_DEVICE_NAME_PATTERN)))
            .arg(Arg::with_name("guid")
                .long("guid")
                .short("g")
                .takes_value(true)
                .required(true)
                .validator(guid_validator)
                .help("guid for ovey device: 64 bit number (please enter as integer to the base of 10)"))
        )
        .subcommand(SubCommand::with_name("delete")
            .display_order(1)
            .about("remove virtual overlay rdma network device")
            .arg(Arg::with_name("name")
                .long("name")
                .short("n")
                .takes_value(true)
                .required(true)
                .validator(dev_name_validator)
                .help(&format!("device name (\"{}\")", DEVICE_NAME_PATTERN)))
        )
        .subcommand(SubCommand::with_name("echo")
            .display_order(2)
            .about("sends a message via netlink and receives a message back")
            .arg(Arg::with_name("value")
                .long("value")
                .takes_value(true)
                .required(true)
                .help("text to send to kernel")))
        .get_matches()
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn sth() {
    }
}