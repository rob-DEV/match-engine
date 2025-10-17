mod client_state;
mod message;
mod parser;
mod process;

use crate::message::GatewayMessage;
use crate::process::engine_msg_in_submitter::initialize_engine_msg_in_message_submitter;
use crate::process::engine_msg_out_receiver::initialize_engine_msg_out_receiver;
use crate::process::gateway_connection_handler::initialize_gateway_session_handler;
use common::domain::messaging::EngineMessage;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::error::Error;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc, Mutex};
use std::{env, thread};

lazy_static! {
    pub static ref GATEWAY_PORT: u16 = env::var("GATEWAY_PORT")
        .unwrap_or("3001".to_owned())
        .parse::<u16>()
        .unwrap();
    pub static ref ENGINE_MSG_IN_PORT: u16 = env::var("ENGINE_PORT")
        .unwrap_or("3000".to_owned())
        .parse::<u16>()
        .unwrap();
    pub static ref ENGINE_MSG_OUT_PORT: u16 = env::var("ENGINE_PORT")
        .unwrap_or("3500".to_owned())
        .parse::<u16>()
        .unwrap();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("--- Initializing Gateway ---");

    let core_ids = core_affinity::get_core_ids()
        .unwrap()
        .into_iter()
        .collect::<Vec<_>>();
    let pinned_msg_in_core = core_ids[1];
    let pinned_msg_out_core = core_ids[2];

    let (client_msg_tx, client_msg_rx): (Sender<GatewayMessage>, Receiver<GatewayMessage>) =
        mpsc::channel();

    let gateway_to_engine_msg_in_tx =
        Arc::new(Mutex::new(HashMap::<u32, Sender<EngineMessage>>::new()));

    let engine_msg_in_thread = thread::spawn(move || {
        core_affinity::set_for_current(pinned_msg_in_core);
        initialize_engine_msg_in_message_submitter(*ENGINE_MSG_IN_PORT, client_msg_rx)
            .expect("failed to initialize engine MSG_IN thread");
    });

    let thread_session_msg_tx_map = gateway_to_engine_msg_in_tx.clone();
    let engine_msg_out_thread = thread::spawn(move || {
        core_affinity::set_for_current(pinned_msg_out_core);
        initialize_engine_msg_out_receiver(*ENGINE_MSG_OUT_PORT, thread_session_msg_tx_map)
            .expect("failed to initialize engine MSG_OUT thread");
    });

    let thread_session_msg_tx_map = gateway_to_engine_msg_in_tx.clone();
    initialize_gateway_session_handler(client_msg_tx, thread_session_msg_tx_map)
        .await
        .expect("failed to initialize gateway session handler");

    engine_msg_out_thread.join().unwrap();
    engine_msg_in_thread.join().unwrap();
    Ok(())
}
