use clap::Parser;

mod server;
mod client;
mod util;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct MatchEngineStartupArgs {
    #[arg(short, long)]
    engine_type: String,
}

fn main() {
    // Initalize match engine as either client or server
    let args = MatchEngineStartupArgs::parse();
    let engine_startup_type =  args.engine_type;

    println!("Intializing match engine as {}", engine_startup_type);

    // Either result or error
    let _ = match engine_startup_type.as_str() {
       "me_server" => server::create_listener(),
       "me_client" => client::connect(),
       _ => panic!("FATAL: BAD engine_startup_type"),
    };
}