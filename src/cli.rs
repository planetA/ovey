//! Utilities to parse the command line

use clap::{ArgMatches, App, SubCommand, Arg};
use regex::Regex;

pub const DEVICE_NAME_PATTERN: &str = "ovey[0-9]+";
pub const PARENT_DEVICE_NAME_PATTERN: &str = "[A-z]+[0-9]+";

/// Parses the args and asserts that required args are in the proper order and format.
pub fn assert_and_get_args<'a>() -> ArgMatches<'a> {

    let dev_name_validator = move |name: String| {
        let regex = Regex::new(DEVICE_NAME_PATTERN).unwrap();
        if regex_is_full_match(&regex, &name) {
            Ok(())
        } else {
            Err(format!("Name of ovey device must match {}", DEVICE_NAME_PATTERN))
        }
    };

    let parent_dev_name_validator = move |name: String| {
        let regex = Regex::new(PARENT_DEVICE_NAME_PATTERN).unwrap();
        if regex_is_full_match(&regex, &name) {
            Ok(())
        } else {
            Err(format!("Name of parent device must match {}", PARENT_DEVICE_NAME_PATTERN))
        }
    };

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
                .validator(dev_name_validator)
                .help(&format!("device name (pattern: {})", DEVICE_NAME_PATTERN)))
            .arg(Arg::with_name("parent")
                .long("parent")
                .short("p")
                .takes_value(true)
                .required(true)
                .validator(parent_dev_name_validator)
                .help(&format!("parent device name (pattern: {})", PARENT_DEVICE_NAME_PATTERN))))
        .subcommand(SubCommand::with_name("delete")
            .display_order(1)
            .about("remove virtual overlay rdma network device")
            .arg(Arg::with_name("name")
                .long("name")
                .short("n")
                .takes_value(true)
                .required(true)
                .validator(dev_name_validator)
                .help(&format!("device name (\"{}\")", DEVICE_NAME_PATTERN))))
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

/// Tests whether a text matches the regex entirely. Otherwise
/// "ovey01515afaf" would be a valid input for "ovey[0-9]+".
fn regex_is_full_match(regex: &Regex, text: &str) -> bool {
    let caps = regex.captures(text);

    // check if there is a match
    if caps.is_none() {
        return false;
    }

    // check there is exatcly one match
    let caps = caps.unwrap();
    if caps.len() != 1 {
        return false;
    }

    // check that match length is text length
    let match_len = caps.get(0).unwrap().range().len();
    match_len == text.len()
}
