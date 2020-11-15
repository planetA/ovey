use regex::Regex;
use libocp::ocp_properties::{DEVICE_NAME_PATTERN, PARENT_DEVICE_NAME_PATTERN};
use librdmautil::GUID_STRING_PATTERN;

pub fn validate_device_name(name: &str) -> Result<(), String> {
    validate_name(DEVICE_NAME_PATTERN, name)
        .map_err(|_| format!("Ovey device name must match the pattern {}", PARENT_DEVICE_NAME_PATTERN))
}

pub fn validate_parent_device_name(name: &str) -> Result<(), String> {
    validate_name(PARENT_DEVICE_NAME_PATTERN, name)
        .map_err(|_| format!("Parent device name must match the pattern {}", PARENT_DEVICE_NAME_PATTERN))
}

pub fn validate_guid(name: &str) -> Result<(), String> {
    validate_name(GUID_STRING_PATTERN, name)
        .map_err(|_| format!("GUID must match the pattern {}", GUID_STRING_PATTERN))
}

fn validate_name(pattern: &str, name: &str) -> Result<(), ()> {
    let regex = Regex::new(pattern).unwrap();
    if regex_is_full_match(&regex, &name) {
        Ok(())
    } else {
        Err(())
    }
}

/// Tests whether a text matches the regex entirely. Otherwise
/// "ovey01515afaf" would be a valid input for "ovey[0-9]+".
fn regex_is_full_match(regex: &Regex, text: &str) -> bool {
    let caps = regex.captures(text);

    // check if one capture has the length of the original string
    // => it's a full match

    // iter through all captures
    caps.iter()
        // iter through all sub captures
        .flat_map(|c| c.iter())
        // unwrap match if exists and get length
        .map(|x| x.map_or(0, |m| m.range().len()))
        // has any the length of the full text?
        .any(|x| x == text.len())
}

#[cfg(test)]
mod tests {

    use super::*;
    use librdmautil::GUID_STRING_PATTERN;

    #[test]
    fn test_match() {
        assert!(validate_device_name("ovey0").is_ok());
        assert!(validate_device_name("ovey123").is_ok());
        assert!(validate_parent_device_name("rxe0").is_ok());
        assert!(validate_guid("abcd:ef12:1314:9000").is_ok());

        assert!(validate_device_name("ovey 123").is_err());
        assert!(validate_parent_device_name("rxe 0").is_err());
        assert!(validate_guid("abcd:ef12:1314::9000").is_err());
        assert!(validate_guid("gbcd:ef12:1314:9000").is_err());
    }

    #[test]
    fn test_regex_is_full_match() {
        assert!(regex_is_full_match(&Regex::new(DEVICE_NAME_PATTERN).unwrap(), "ovey0"), "'ovey0' must be valid");
        assert!(!regex_is_full_match(&Regex::new(DEVICE_NAME_PATTERN).unwrap(), "ovey0 "), "'ovey0 ' must be invalid");
        assert!(regex_is_full_match(&Regex::new(PARENT_DEVICE_NAME_PATTERN).unwrap(), "rxe0"), "'rxe0' must be valid");
        assert!(!regex_is_full_match(&Regex::new(PARENT_DEVICE_NAME_PATTERN).unwrap(), "rxe0 "), "'rxe0 ' must be invalid");
        assert!(regex_is_full_match(&Regex::new(GUID_STRING_PATTERN).unwrap(), "dead:beef:dead:beef"), "'dead:beef:dead:beef' must be valid");
    }
}
