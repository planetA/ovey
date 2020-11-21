use regex::Regex;
use libocp::ocp_properties::{DEVICE_NAME_PATTERN, PARENT_DEVICE_NAME_PATTERN};
use liboveyutil::guid::GUID_STRING_PATTERN;
use liboveyutil::regex::regex_is_full_match;

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

#[cfg(test)]
mod tests {

    use super::*;

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
}
