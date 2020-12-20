use serde::{Deserialize, Serialize};
use std::collections::{HashMap};
use std::fs::File;
use std::io::Read;
use liboveyutil::types::VirtualNetworkIdType;

lazy_static::lazy_static! {
    pub static ref CONFIG: InitDataConfiguration = {
        let opt: Result<InitDataConfiguration, std::io::Error> = setup_init_config();
        if opt.is_err() {
            panic!("Ovey daemon needs an init configuration! {:#?}", opt.unwrap_err())
        }
        let cfg = opt.unwrap();

        cfg
    };
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InitDataConfiguration {
    /// Mapping from virtual network id to url / REST-Service of the coordinator.
    coordinators: HashMap<VirtualNetworkIdType, String>
}

impl InitDataConfiguration {
    pub fn coordinators(&self) -> &HashMap<VirtualNetworkIdType, String> {
        &self.coordinators
    }
}

fn setup_init_config() -> Result<InitDataConfiguration, std::io::Error> {
    // TODO ENv Var
    let mut file = File::open("../ovey_daemon/res/ovey_daemon.conf.json")?;
    let mut file_content = String::new();
    let _length = file.read_to_string(&mut file_content)?;

    let config: InitDataConfiguration = serde_json::from_str(&file_content)?;

    Ok(config)
}