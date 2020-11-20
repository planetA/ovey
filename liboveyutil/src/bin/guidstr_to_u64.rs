use liboveyutil::guid::guid_string_to_u64;

fn main() {
    let input = "dead:beef:0bad:f00d";
    let output = guid_string_to_u64(input);

    println!("guid string to u64 (big endian):");
    println!("  {} =>", input);
    println!("  {} (big endian)", u64::from_be(output));
    println!("  {} (little endian)", u64::from_le(output));
}


