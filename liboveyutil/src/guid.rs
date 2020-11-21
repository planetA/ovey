/// String representation of a guid. e.g.: "0000:0000:0000:0000" or "abc0:afaf:34b5:0015".
/// A hex string splitted into groups of four hex ditits connected with '.'.
pub const GUID_STRING_PATTERN: &str = "[AaBbCcDdEeFf0-9]{4}(:[AaBbCcDdEeFf0-9]{4}){3}";

/// Transforms the guid string representation into a u64.
/// The string representation is basically the hex string.
/// The additional delimiters (':') must be removed and the
/// string then interpreted as number. That's is.
///
/// See `GUID_STRING_PATTERN`.
///
/// * `repr` String representation of guid
pub fn guid_string_to_u64(repr: &str) -> u64 {
    let hexstr = repr.replace(":", "");
    u64::from_str_radix(&hexstr, 16).unwrap()
}

/// Converts a guid to its string representation (like it is done in libibverbs).
/// The string representation is a hex string with the exception that the string
/// is splitted into groups of four hex digits connected with ':'.
pub fn guid_u64_to_string(guid: u64) -> String {
    let p0 = guid >>  0 & 0xffff;
    let p1 = guid >> 16 & 0xffff;
    let p2 = guid >> 32 & 0xffff;
    let p3 = guid >> 48 & 0xffff;

    format!("{:04x}:{:04x}:{:04x}:{:04x}", p3, p2, p1, p0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::endianness::Endianness;

    #[test]
    fn test_guid_string_to_ube64() {
        // input host endianness
        let input_he = "dead:beef:0000:0000";
        let expected_he = 0xdead_beef_0000_0000_u64;
        let actual_he = guid_string_to_u64(input_he);
        assert_eq!(expected_he, actual_he);
    }

    #[test]
    fn test_guid_he_to_string() {
        // input host endianness
        let input_he = 0xdead_beef_0000_0000_u64;
        let expected_he = "dead:beef:0000:0000";
        let actual_he = guid_u64_to_string(input_he);
        assert_eq!(expected_he, actual_he);
    }

    /// u64 to string and backwards with an intermediate step for
    /// big endian transformation. This test is important for
    /// little endian platforms like x86.
    #[test]
    fn test_be_transformation_step() {
        let input    = "dead:beef:0000:0000";
        let expected = "dead:beef:0000:0000";

        let input_he = guid_string_to_u64(input);

        // pretend it gets stored in kernel (big endian u64)
        let stored_guid_u64_be = Endianness::u64he_to_u64be(input_he);
        // pretend we load the data from kernel into host endianness
        let input_he = Endianness::u64be_to_u64he(stored_guid_u64_be);

        let actual = guid_u64_to_string(input_he);

        assert_eq!(expected, actual)
    }
}
