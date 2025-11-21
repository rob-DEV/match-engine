use crate::algorithm::fifo_match_strategy::FifoMatchStrategy;
use crate::algorithm::match_strategy::MatchStrategy;
use crate::algorithm::pro_rata_match_strategy::ProRataMatchStrategy;

#[derive(Debug)]
pub struct EngineConfig {
    pub msg_in_port: u16,
    pub msg_out_port: u16,

    pub instrument: String,
    pub instrument_id: u32,
    pub match_strategy: Box<dyn MatchStrategy>,
}

impl EngineConfig {
    pub fn load(json_path: &str) -> Self {
        let raw_config = EngineConfigRaw::load_raw_engine_config(json_path);

        EngineConfig {
            msg_in_port: raw_config.msg_in_port,
            msg_out_port: raw_config.msg_out_port,
            instrument: raw_config.instrument,
            instrument_id: raw_config.instrument_id,
            match_strategy: raw_config_match_strategy(&raw_config.match_strategy),
        }
    }
}
fn raw_config_match_strategy(match_strategy: &str) -> Box<dyn MatchStrategy> {
    match match_strategy {
        "FIFO" => Box::new(FifoMatchStrategy::new()),
        "PRO_RATA" => Box::new(ProRataMatchStrategy::new()),
        _ => panic!("Unknown match_strategy"),
    }
}

use serde::Deserialize;
use serde_json::Value;
use std::fs;

#[derive(Deserialize)]
struct EngineConfigRaw {
    pub msg_in_port: u16,
    pub msg_out_port: u16,
    pub instrument: String,
    pub instrument_id: u32,
    pub match_strategy: String,
}

impl EngineConfigRaw {
    fn load_raw_engine_config(json_config_path: &str) -> EngineConfigRaw {
        let data = fs::read_to_string(json_config_path).expect("Error reading json file");
        let root: Value = serde_json::from_str(&data).unwrap();

        let engine = "engine";

        let prefixed = root
            .get("engine")
            .ok_or_else(|| format!("Prefix '{}' not found in config", engine))
            .unwrap();

        serde_json::from_value(prefixed.clone()).unwrap()
    }
}
