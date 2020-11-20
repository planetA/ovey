use std::fs::File;
use std::io::Read;
use ovey_coordinator::rest::structs::InitDataConfiguration;

lazy_static::lazy_static! {
    pub static ref CONFIG: InitDataConfiguration = {
        let opt: Result<InitDataConfiguration, std::io::Error> = setup_init_config();
        if opt.is_err() {
            panic!("Ovey coordinator needs an init configuration! {:#?}", opt.unwrap_err())
        }
        let cfg = opt.unwrap();

        // register all networks
        cfg.networks().keys().for_each(|key| {
            crate::db::db_register_network(key.to_owned()).unwrap();
        });

        cfg
    };
}

fn setup_init_config() -> Result<InitDataConfiguration, std::io::Error> {
    // TODO ENv Var
    let mut file = File::open("../ovey_coordinator/res/ovey_coordinator.conf.json")?;
    let mut file_content = String::new();
    let _length = file.read_to_string(&mut file_content)?;

    let config: InitDataConfiguration = serde_json::from_str(&file_content)?;

    Ok(config)
}