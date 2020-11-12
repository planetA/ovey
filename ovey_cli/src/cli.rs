//! Utilities to parse the command line

use clap::{ArgMatches, App, SubCommand, Arg};
use regex::Regex;
use libocp::ocp_properties::{DEVICE_NAME_PATTERN, PARENT_DEVICE_NAME_PATTERN};
use librdmautil::GUID_STRING_PATTERN;

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

    let guid_validator = move |name: String| {
        let regex = Regex::new(GUID_STRING_PATTERN).unwrap();
        if regex_is_full_match(&regex, &name) {
            Ok(())
        } else {
            Err(format!("Format of guid is invalid. Example: '0a00:27ff:fec7:499a', Pattern: {}", GUID_STRING_PATTERN))
        }
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

/// Tests whether a text matches the regex entirely. Otherwise
/// "ovey01515afaf" would be a valid input for "ovey[0-9]+".
fn regex_is_full_match(regex: &Regex, text: &str) -> bool {
    let caps = regex.captures(text);

    // check if one capture has the length of the original string
    // => it's a full match

    // iter through all captures
    caps.iter()
        // iter through all sub saptures
        .flat_map(|c| c.iter())
        // unwrap match if exists and get length
        .map(|x| x.map_or(0, |m| m.range().len()))
        // has any the length of the full text?
        .any(|x| x == text.len())
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_regex_is_full_match() {
        assert!(regex_is_full_match(&Regex::new(DEVICE_NAME_PATTERN).unwrap(), "ovey0"), "'ovey0' must be valid");
        assert!(!regex_is_full_match(&Regex::new(DEVICE_NAME_PATTERN).unwrap(), "ovey0 "), "'ovey0 ' must be invalid");
        assert!(regex_is_full_match(&Regex::new(PARENT_DEVICE_NAME_PATTERN).unwrap(), "rxe0"), "'rxe0' must be valid");
        assert!(!regex_is_full_match(&Regex::new(PARENT_DEVICE_NAME_PATTERN).unwrap(), "rxe0 "), "'rxe0 ' must be invalid");
        assert!(regex_is_full_match(&Regex::new(GUID_STRING_PATTERN).unwrap(), "dead:beef:dead:beef"), "'dead:beef:dead:beef' must be valid");
    }
}