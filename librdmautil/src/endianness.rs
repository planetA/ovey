//! Utility functions to cope with endianess. This is necessary because OFED (Linux RDMA stack)
//! expects guid to be stored in big endian format.

#[derive(Debug)]
pub enum Endianness {
    BigEndian,
    LittleEndian
}

impl Endianness {
    pub const fn get_system_endianess() -> Endianness {
        let num: u16 = 0b1000_0000_0000_0000; // 2 bytes
        // will reorder the bytes on a little endian machine
        // and leave it the same way on a big endian machine
        let num_be = u16::from_be(num);

        if num_be == 0b0000_0000_1000_0000 { // byte order changed
            // did a conversion from big endian to little endian
            Endianness::LittleEndian
        } else {
            // did a conversion from big endian to little endian
            Endianness::BigEndian
        }
    }

    /// Returns if the current system uses big endian byte order.
    pub const fn system_is_be() -> bool {
        if let Endianness::BigEndian = Endianness::get_system_endianess() {
            true
        } else {
            false
        }
    }

    /// Returns if the current system uses little endian byte order.
    pub const fn system_is_le() -> bool {
        !Endianness::system_is_be()
    }

    /// Converts a u64 number from host endian format to u64 big endian format.
    pub fn u64he_to_u64be(u64he: u64) -> u64 {
        if Endianness::system_is_le() {
            // will trigger a reorder of the bytes if current system is little endian
            u64::from_be(u64he)
        } else {
            u64he
        }
    }

    /// Converts a u64 number from big endian format to host endian format.
    pub fn u64be_to_u64he(u64be: u64) -> u64 {
        if Endianness::system_is_le() {
            // will trigger a reorder of the bytes if current system is little endian
            u64::from_be(u64be)
        } else {
            u64be
        }
    }

    pub fn change(val: u64) -> u64 {
        if Endianness::system_is_le() {
            // will trigger a reorder of the bytes if current system is little endian
            u64::from_be(val)
        } else {
            // will trigger a reorder of the bytes if current system is big endian
            u64::from_le(val)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// I have soo much struggles with endianness aware code that
    /// I want to ensure such basic things.. to understand what's happing...
    #[test]
    fn test_system_endianness() {
        let bin: u64 = 0b11110000_00000000_00000000_00000000_00000000_00000000_00000000_00000000;
        let hex: u64 = 0xf0_00_00_00_00_00_00_00;
        let decimal: u64 = 17293822569102704640;

        assert_eq!(bin, hex);
        assert_eq!(decimal, hex);
    }

    #[test]
    fn test_endianness() {
        let endianess = Endianness::get_system_endianess();
        println!("get_system_endianess: {:#?}", endianess);

        // 8 bytes: not the memory representation but the compiler representation
        // on x86 this will be stored in little endian order
        let input = 0xff00_0000_0000_0000_u64;
        let input_opposite_endianness = 0x0000_0000_0000_00ff_u64;

        let actual_opposite;
        let actual_same;
        if Endianness::system_is_be() {
            // then big endian to little endian
            actual_opposite = Endianness::u64be_to_u64he(input);
            actual_same = Endianness::u64he_to_u64be(input);
        } else {
            actual_opposite = Endianness::u64he_to_u64be(input);
            actual_same = Endianness::u64be_to_u64he(input_opposite_endianness);
        }

        assert_eq!(actual_opposite, input_opposite_endianness, "Byte endianness must be changed!");
        assert_eq!(actual_same, input, "Byte endianness must be the same!");
    }
}
