use std::io::Write;
use clap::{arg, Arg, ArgAction, Command};

fn main() -> Result<(), String> {
    loop {
        let line = read_input_line()?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        match process_input_line(line) {
            Ok(quit) => {
                if quit {
                    break;
                }
            }
            Err(err) => {
                write!(std::io::stdout(), "{err}\n").map_err(|e| e.to_string())?;
                std::io::stdout().flush().map_err(|e| e.to_string())?;
            }
        }
    }

    Ok(())
}

fn process_input_line(line: &str) -> Result<bool, String> {
    let args = shlex::split(line).ok_or("error: Invalid quoting")?;
    let matches = cli()
        .try_get_matches_from(args)
        .map_err(|e| e.to_string())?;
    match matches.subcommand() {
        Some(("connect", _matches)) => {
            write!(std::io::stdout(), "Pong\n").map_err(|e| e.to_string())?;
            std::io::stdout().flush().map_err(|e| e.to_string())?;
        }
        Some(("new", _matches)) => {
            write!(std::io::stdout(), "Pong\n").map_err(|e| e.to_string())?;
            std::io::stdout().flush().map_err(|e| e.to_string())?;
        }
        Some(("cancel", _matches)) => {
            write!(std::io::stdout(), "Pong\n").map_err(|e| e.to_string())?;
            std::io::stdout().flush().map_err(|e| e.to_string())?;
        }
        Some(("quit", _matches)) => {
            write!(std::io::stdout(), "Exiting ...\n").map_err(|e| e.to_string())?;
            std::io::stdout().flush().map_err(|e| e.to_string())?;
            return Ok(true);
        }
        Some((name, _matches)) => unimplemented!("{name}"),
        None => unreachable!("subcommand required"),
    }

    Ok(false)
}

fn cli() -> Command {
    Command::new("repl")
        .multicall(true)
        .arg_required_else_help(true)
        .subcommand_required(true)
        .subcommand_value_name("COMMAND")
        .subcommand_help_heading("Usage")
        .subcommand(
            Command::new("connect")
                .about("Establish the fix connection")
                .arg(arg!(<HOST> "Gateway host"))
                .arg(arg!(<PORT> "Gateway Port"))
        )
        .subcommand(
            Command::new("new")
                .about("Send an order to the engine")
                .arg(arg!(<ACTION> "B/O"))
                .arg(arg!(<QTY> "Quantity"))
                .arg(arg!(<PX> "Price"))

        )
        .subcommand(
            Command::new("cancel")
                .about("Cancel an order")
                .arg(arg!(<ORDER_ID> "Order to cancel"))
        )
        .subcommand(
            Command::new("quit")
                .alias("exit")
                .about("Exit the tool")
        )
}

fn read_input_line() -> Result<String, String> {
    write!(std::io::stdout(), "$ ").map_err(|e| e.to_string())?;
    std::io::stdout().flush().map_err(|e| e.to_string())?;
    let mut buffer = String::new();
    std::io::stdin()
        .read_line(&mut buffer)
        .map_err(|e| e.to_string())?;
    Ok(buffer)
}