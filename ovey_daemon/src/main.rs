use ovey_coordinator::rest::structs::VirtualizedDeviceInputBuilder;

fn main() {

}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn builder_works() {
        // see https://crates.io/crates/derive_builder
        let foo = VirtualizedDeviceInputBuilder::default()
            .virtual_device_guid_string("1000:0000:0000:0000")
            .physical_device_guid_string("3000:0000:0000:0000")
            .build()
            .unwrap();
        println!("{:#?}", foo);
    }

}