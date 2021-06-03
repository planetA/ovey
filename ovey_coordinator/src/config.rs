use std::fs::File;
use std::io::Read;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use liboveyutil::types::GuidString;

#[derive(Serialize, Deserialize, Debug)]
pub struct InitDataConfiguration {
    networks: HashMap<Uuid, Vec<GuidString>>
}

/// The name of the env variable where the coordinator
/// expects the configuration file. Can be absolute or relative.
pub const ENV_VAR_CONFIG_FILE: &str = "OVEY_COORDINATOR_CFG";

lazy_static::lazy_static! {
    pub static ref CONFIG: InitDataConfiguration = {
        let opt: Result<InitDataConfiguration, std::io::Error> = setup_init_config();
        if opt.is_err() {
            panic!("Ovey coordinator needs an init configuration! {:#?}", opt.unwrap_err())
        }
        let cfg = opt.unwrap();

        // register all networks
        cfg.networks.keys().for_each(|key| {
            // crate::db::db_register_network(key.to_owned()).unwrap();
        });

        cfg
    };
}

fn setup_init_config() -> Result<InitDataConfiguration, std::io::Error> {
    // this path works when the binary is executed from the IDE / via cargo run
    let default_path = "./ovey_coordinator/res/ovey_coordinator.conf.json".to_string();
    let path = match std::env::var(ENV_VAR_CONFIG_FILE) {
        Ok(path) => {path}
        Err(_) => {default_path}
    };
    eprintln!("Using config file: '{}'", path);
    let mut file = File::open(path)?;
    let mut file_content = String::new();
    let _length = file.read_to_string(&mut file_content)?;

    let config: InitDataConfiguration = serde_json::from_str(&file_content)?;

    Ok(config)
}
