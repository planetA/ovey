//! Utily functions related to ib devices and verbs.

use crate::endianness::Endianness;

/// String representation of a guid. e.g.: "0000:0000:0000:0000" or "abc0:afaf:34b5:0015"
pub const GUID_STRING_PATTERN: &str = "[AaBbCcDdEeFf0-9]{4}(:[AaBbCcDdEeFf0-9]{4}){3}";

pub mod endianness;

/// Returns a big endian encoded u64 from a "big endian" guid hex string.
/// See `GUID_STRING_PATTERN`.
///
/// * `repr` String representation in big endian order
pub fn guid_string_to_ube64(repr: &str) -> u64 {
    let hexstr = repr.replace(":", "");
    // is already in big endian because string is
    let guid_be = u64::from_str_radix(&hexstr, 16).unwrap();
    guid_be
}

/// Converts a guid in host endianess to big endian and calls
/// `guid_be_to_string()`
pub fn guid_he_to_string(mut guid_he: u64) -> String {
    guid_be_to_string(Endianness::u64he_to_u64be(guid_he))
}

/// Converts a guid in big endian format to it's string representation.
pub fn guid_be_to_string(guid_be: u64) -> String {
    let p0 = guid_be >>  0 & 0xffff;
    let p1 = guid_be >> 16 & 0xffff;
    let p2 = guid_be >> 32 & 0xffff;
    let p3 = guid_be >> 48 & 0xffff;

    format!("{:04x}:{:04x}:{:04x}:{:04x}", p3, p2, p1, p0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_guid_string_to_ube64() {
        let guid_he = 11117637053157146634;
        let guid_be = Endianness::u64he_to_u64be(guid_he);
        let expected = guid_be;
        let input = "0a00:27ff:fec7:499a";
        let actual = guid_string_to_ube64(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_guid_string_to_ube64_2() {
        let expected_he = 0xdead_beef_0000_0000_u64;
        let expected_be = Endianness::u64he_to_u64be(expected_he);
        let input = "dead:beef:0000:0000";
        let actual = guid_string_to_ube64(input);

        assert_eq!(expected_he, actual, "{:x} != {:x}", expected_be, actual);
    }

    #[test]
    fn test_guid_he_to_string() {
        let input_he = 0xdead_beef_0000_0000_u64;
        let input_be = Endianness::u64he_to_u64be(input_he);
        let expected = "dead:beef:0000:0000";
        let actual = guid_he_to_string(input_be);
        assert_eq!(expected, actual);
    }
}