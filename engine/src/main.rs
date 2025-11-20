use crate::algorithm::match_strategy::MatchStrategy;
use crate::engine::engine_config::EngineConfig;
use crate::engine::match_server::MatchServer;
use lazy_static::lazy_static;
use std::env;
use std::error::Error;

mod algorithm;
mod book;
mod domain;
mod engine;
mod process;

lazy_static! {
    pub static ref ENGINE_MSG_IN_PORT: u16 = env::var("ENGINE_PORT")
        .unwrap_or("3000".to_owned())
        .parse::<u16>()
        .unwrap();
    pub static ref ENGINE_MSG_OUT_PORT: u16 = env::var("ENGINE_PORT")
        .unwrap_or("3500".to_owned())
        .parse::<u16>()
        .unwrap();
}

fn main() {
    println!("--- Initializing Match Engine ---");

    let engine_config =
        EngineConfig::load("/home/robert/dev/match-engine/config/engine_btc_usd.json");

    let mut match_server = MatchServer::new(engine_config);
    match_server.run();
}
