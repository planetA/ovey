/// Parse LID string to number for internal representation
pub fn lid_string_to_u16(repr: &str) -> u16 {
    repr.parse::<u16>().unwrap()
}

/// Converts an LID to its string representation (like it is done in libibverbs).
/// The string representation is a hex string with the exception that the string
/// is splitted into groups of four hex digits connected with ':'.
pub fn lid_u16_to_string(lid: u16) -> String {
    format!("0x{:04x}", lid)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_guid_string_to_ube64() {
        // input host endianness
        let input_he = "0xdead";
        let expected_he = 0xdead_u16;
        let actual_he = lid_string_to_u16(input_he);
        assert_eq!(expected_he, actual_he);
    }

    #[test]
    fn test_guid_he_to_string() {
        // input host endianness
        let input_he = 0xdead_u16;
        let expected_he = "0xdead";
        let actual_he = lid_u16_to_string(input_he);
        assert_eq!(expected_he, actual_he);
    }
}
