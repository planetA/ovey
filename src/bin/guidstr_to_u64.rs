use rdma_ovey::verbs::guid_string_to_ube64;

fn main() {
    let input = "0a00:27ff:fec7:499a";
    let output = guid_string_to_ube64(input);

    println!("guid string to u64 (big endian):");
    println!("  {} =>", input);
    println!("  {} (big endian)", u64::from_be(output));
    println!("  {} (little endian)", u64::from_le(output));
}


