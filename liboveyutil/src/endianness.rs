//! Utility functions to cope with endianess. This is necessary because OFED (Linux RDMA stack)
//! expects guid to be stored in big endian format. What we do is pretty simple.
//! We always handle the 64 bit number "regular"/as is/in native endian order.
//! When we store it in the kernel we change the order to big endian
//! (only necessary on little endian system).

#[derive(Debug)]
pub enum Endianness {
    BigEndian,
    LittleEndian
}

impl Endianness {
    pub const fn get_system_endianness() -> Endianness {
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
        if let Endianness::BigEndian = Endianness::get_system_endianness() {
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
    /// This is needed when we want to store the data inside the kernel module.
    /// The kernel module stores the information about guid in u64 big endian format.
    pub fn u64he_to_u64be(u64he: u64) -> u64 {
        if Endianness::system_is_le() {
            // will trigger a reorder of the bytes if current system is little endian
            u64::from_be(u64he)
        } else {
            u64he
        }
    }

    /// Converts a u64 number from big endian format to host endian format.
    /// This is needed when we want to load the data from the kernel module.
    /// The kernel module stores the information about guid in u64 big endian format.
    pub fn u64be_to_u64he(u64be: u64) -> u64 {
        if Endianness::system_is_le() {
            // will trigger a reorder of the bytes if current system is little endian
            u64::from_be(u64be)
        } else {
            u64be
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_endianness() {
        let endianess = Endianness::get_system_endianness();
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
