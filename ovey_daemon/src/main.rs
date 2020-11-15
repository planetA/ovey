use ovey_coordinator::rest::structs::*;
use config::CONFIG;
use ovey_coordinator::OVEY_COORDINATOR_PORT;
use std::process::exit;

mod config;

fn main() {
    println!("Ovey daemon started. Got the following initial configuration:");
    println!("{:#?}", *CONFIG);

    let mut actual_up = 0;
    let expected_up = CONFIG.coordinators().len();

    if expected_up == 0 {
        eprintln!("There is not a single Ovey coordinator configured.");
        exit(-1);
    }

    // check all hosts are available
    CONFIG.coordinators().keys().for_each(|network| {
        // url with scheme, like http://localhost or https://foobar.com/
        // we make a Request to "/" and check if 200 OK is a response.
        let host = CONFIG.coordinators().get(network).unwrap();
        // http://localhost:13337
        let mut host = host.to_owned();
        host.push_str(&format!(":{}", OVEY_COORDINATOR_PORT));

        let resp = reqwest::blocking::get(&host);
        if resp.is_err() {
            eprintln!("Ovey coordinator on configured host '{}' IS NOT UP.", &host);
        } else {
            println!("Ovey Coordinator @ {} IS UP", host);

            let resp = resp.unwrap();
            let resp = resp.json::<AllNetworksDtoType>();

            // check if the Ovey coordinator also has the right ovey network configured
            if let Result::Ok(json) = resp {
                if json.contains_key(network) {
                    actual_up += 1;
                } else {
                    eprintln!(
                        "Ovey coordinator on configured host '{}' does not know about network '{}'!",
                        &host,
                        network
                    );
                }
            } else {
                eprintln!("Ovey coordinator @ host '{}' sent invalid response.", &host);
            }
        }
    });

    if actual_up != expected_up {
        eprintln!("WARNING: Not all Ovey coordinators are available.");
    } else {
        println!("INFO: All Ovey coordinators are available.")
    }
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