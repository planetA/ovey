use librdmautil::guid_be_to_string;

fn main() {
    // input big endian
    let input_be = 11117637053157146634_u64;

    let output = guid_be_to_string(input_be);
    println!("guid u64 (big endian) to str:");
    println!("  {} (big endian) =>", input_be);
    println!("  {} ", output);
}


