use serde::{Deserialize, Serialize};
use std::collections::{HashMap};
use std::fs::File;
use std::io::Read;
use uuid::Uuid;
use ovey_coordinator::OVEY_COORDINATOR_PORT;

/// The name of the env variable where the daemon
/// expects the configuration file. Can be absolute or relative.
pub const ENV_VAR_CONFIG_FILE: &str = "OVEY_DAEMON_CFG";

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
    coordinators: HashMap<Uuid, String>,
    check_coordinators: bool
}

impl InitDataConfiguration {
    pub fn get_coordinator(&self, network: &Uuid) -> Option<String> {
        let host = self.coordinators.get(network)?;
        Some(format!("{}:{}", host, OVEY_COORDINATOR_PORT))
    }
}

fn setup_init_config() -> Result<InitDataConfiguration, std::io::Error> {
    // this path works when the binary is executed from the IDE / via cargo run
    let default_path ="./ovey_daemon/res/ovey_daemon.conf.json".to_string();
    let path = match std::env::var(ENV_VAR_CONFIG_FILE) {
        Ok(path) => {path}
        Err(_) => {default_path}
    };
    info!("Using config file: '{}'", path);
    let mut file = File::open(path)?;
    let mut file_content = String::new();
    let _length = file.read_to_string(&mut file_content)?;

    let config: InitDataConfiguration = serde_json::from_str(&file_content)?;

    Ok(config)
}
