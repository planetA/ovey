//! Utily functions related to ib devices and verbs.

/// String representation of a guid. e.g.: "0000:0000:0000:0000" or "abc0:afaf:34b5:0015"
pub const GUID_STRING_PATTERN: &str = "[AaBbCcDdEeFf0-9]{4}(:[AaBbCcDdEeFf0-9]{4}){3}";

/// Returns a big endian encoded u64 from a guid string.
/// See `GUID_STRING_PATTERN`. NOTE: Please be aware that
/// this value must first be transformed to "host endianness"
/// if this should be transferred to kernel via OCP.
///
/// * `repr` String representation in big endian order
pub fn guid_string_to_ube64(repr: &str) -> u64 {
    let mut split = repr.split(":");
    let gr0 = split.next().unwrap();
    let gr1 = split.next().unwrap();
    let gr2 = split.next().unwrap();
    let gr3 = split.next().unwrap();

    // each value only takes 16 bits, because each string only contains
    // 4 hex digits
    let p0 = u64::from_str_radix(gr0, 16).unwrap();
    let p1 = u64::from_str_radix(gr1, 16).unwrap();
    let p2 = u64::from_str_radix(gr2, 16).unwrap();
    let p3 = u64::from_str_radix(gr3, 16).unwrap();

    // construct u64 in big endian order
    let mut guid_be = 0;
    guid_be |= p3 << 0;
    guid_be |= p2 << 16;
    guid_be |= p1 << 32;
    guid_be |= p0 << 48;

    guid_be
}

pub fn guid_to_string(guid: u64) -> String {
    let guid = u64::from_be(guid);
    let p0 = guid >>  0 & 0xffff;
    let p1 = guid >> 16 & 0xffff;
    let p2 = guid >> 32 & 0xffff;
    let p3 = guid >> 48 & 0xffff;

    format!("{:04x}:{:04x}:{:04x}:{:04x}", p3, p2, p1, p0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_guid_string_to_ube64() {
        let expected = 720619920823896474_u64;
        let expected_2 = 0x0a00_27ff_fec7_499a_u64;
        let input = "0a00:27ff:fec7:499a";
        let actual = guid_string_to_ube64(input);
        assert_eq!(expected, expected_2); // just check my assumption is wright
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_guid_string_to_ube64_2() {
        let expected = 0xdead_beef_0000_0000_u64;
        let input = "dead:beef:0000:0000";
        let actual = guid_string_to_ube64(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_guid_to_string() {
        let input = 11117637053157146634_u64;
        let expected = "0a00:27ff:fec7:499a";
        let actual = guid_to_string(input);
        assert_eq!(expected, actual);
    }
}