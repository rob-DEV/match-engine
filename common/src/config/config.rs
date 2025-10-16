use config::{Config, File};
use std::collections::HashMap;
use std::path::Path;

pub fn load_engine_config() -> HashMap<String, String> {
    let settings = Config::builder()
        .add_source(File::from(Path::new("/home/robert/dev/match-engine/config/config.json")))
        .build()
        .unwrap();

    let config_map = settings
        .try_deserialize::<HashMap<String, String>>()
        .unwrap();

    println!(
        "\n{:?} \n\n-----------",
        config_map
    );

    config_map
}

