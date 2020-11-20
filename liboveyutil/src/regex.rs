use regex::Regex;

/// Tests whether a text matches the regex entirely. Otherwise
/// "ovey01515afaf" would be a valid input for "ovey[0-9]+" because
/// the Regex matches a substring of the input.
pub fn regex_is_full_match(regex: &Regex, text: &str) -> bool {
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
    use libocp::ocp_properties::{DEVICE_NAME_PATTERN, PARENT_DEVICE_NAME_PATTERN};
    use crate::guid::GUID_STRING_PATTERN;

    #[test]
    fn test_regex_is_full_match() {
        assert!(regex_is_full_match(&Regex::new(DEVICE_NAME_PATTERN).unwrap(), "ovey0"), "'ovey0' must be valid");
        assert!(!regex_is_full_match(&Regex::new(DEVICE_NAME_PATTERN).unwrap(), "ovey0 "), "'ovey0 ' must be invalid");
        assert!(regex_is_full_match(&Regex::new(PARENT_DEVICE_NAME_PATTERN).unwrap(), "rxe0"), "'rxe0' must be valid");
        assert!(!regex_is_full_match(&Regex::new(PARENT_DEVICE_NAME_PATTERN).unwrap(), "rxe0 "), "'rxe0 ' must be invalid");
        assert!(regex_is_full_match(&Regex::new(GUID_STRING_PATTERN).unwrap(), "dead:beef:dead:beef"), "'dead:beef:dead:beef' must be valid");
    }
}