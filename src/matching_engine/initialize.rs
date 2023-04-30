use clap::Parser;

use super::network;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct MatchEngineStartupArgs {
    #[arg(short, long)]
    engine_type: String,
}

pub fn initialize() {
    let args = MatchEngineStartupArgs::parse();
    let engine_startup_type =  args.engine_type;

    println!("Intializing match engine");

    let _ = match engine_startup_type.as_str() {
       "server" => network::create_listener(),
       "client" => network::establish_connection(),
       _ => panic!("FATAL: BAD engine_startup_type"),
    };
}
