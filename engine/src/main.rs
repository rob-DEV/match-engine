use crate::internal::order::Order;
use common::config::config::load_engine_config;
use common::network::multicast::multicast_udp_socket;
use common::domain::messaging::OutboundEngineMessage;
use common::util::time::wait_50_milli;
use lazy_static::lazy_static;
use std::error::Error;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::{env, thread};
use crate::threaded_processors::msg_in_processor::{engine_msg_out_to_multicast, multicast_receiver_to_engine_msg_in};

mod engine;
mod internal;
mod memory;
mod book;
mod threaded_processors;

lazy_static! {
    pub static ref ENGINE_MSG_IN_PORT: u16 = env::var("ENGINE_PORT").unwrap_or("3000".to_owned()).parse::<u16>().unwrap();
    pub static ref ENGINE_MSG_OUT_PORT: u16 = env::var("ENGINE_PORT").unwrap_or("3500".to_owned()).parse::<u16>().unwrap();
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("--- Initializing Match Engine ---");
    let config = load_engine_config();

    let (engine_msg_out_tx, engine_msg_out_rx): (Sender<OutboundEngineMessage>, Receiver<OutboundEngineMessage>) = mpsc::channel();
    let (order_entry_tx, order_entry_rx): (Sender<Order>, Receiver<Order>) = mpsc::channel();

    // Core OE and Match Thread
    let engine_thread = thread::spawn(move || {
        let mut match_engine = engine::match_engine::MatchEngine::new(config.get("symbol").unwrap().to_owned(), config.get("isin").unwrap().to_owned());
        match_engine.run(order_entry_rx, engine_msg_out_tx);
    });

    wait_50_milli();

    // MULTICAST-> ENGINE MSG_IN
    let engine_msg_in_thread = thread::spawn(|| {
        initialize_engine_msg_in_thread(order_entry_tx);
    });

    wait_50_milli();

    // ENGINE MSG_OUT -> MULTICAST
    let engine_msg_out_thread = thread::spawn(|| {
        initialize_engine_msg_thread(engine_msg_out_rx)
    });

    engine_thread.join().unwrap();
    engine_msg_out_thread.join().unwrap();
    engine_msg_in_thread.join().unwrap();

    Ok(())
}

fn initialize_engine_msg_in_thread(order_entry_tx: Sender<Order>) -> ! {
    println!("Initializing Engine MSG_IN multicast on port {}", *ENGINE_MSG_IN_PORT);
    let msg_in_socket = multicast_udp_socket(*ENGINE_MSG_IN_PORT, true);
    multicast_receiver_to_engine_msg_in(&msg_in_socket, &order_entry_tx);
}

fn initialize_engine_msg_thread(rx: Receiver<OutboundEngineMessage>) -> ! {
    println!("Initializing Engine MSG_OUT multicast on port {}", *ENGINE_MSG_OUT_PORT);
    let msg_out_socket = multicast_udp_socket(*ENGINE_MSG_OUT_PORT, false);
    engine_msg_out_to_multicast(&rx, &msg_out_socket)
}