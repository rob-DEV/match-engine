use crate::process::match_thread::initialize_match_thread;
use crate::process::msg_in_thread::initialize_engine_msg_in_thread;
use crate::process::msg_out_thread::initialize_engine_msg_out_thread;
use common::domain::messaging::SequencedEngineMessage;
use common::domain::order::Order;
use common::util::time::wait_50_milli;
use lazy_static::lazy_static;
use std::error::Error;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::{env, thread};

mod engine;
mod book;
mod process;
mod algorithm;

lazy_static! {
    pub static ref ENGINE_MSG_IN_PORT: u16 = env::var("ENGINE_PORT").unwrap_or("3000".to_owned()).parse::<u16>().unwrap();
    pub static ref ENGINE_MSG_OUT_PORT: u16 = env::var("ENGINE_PORT").unwrap_or("3500".to_owned()).parse::<u16>().unwrap();
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("--- Initializing Match Engine ---");

    let (engine_msg_out_tx, engine_msg_out_rx): (Sender<SequencedEngineMessage>, Receiver<SequencedEngineMessage>) = mpsc::channel();
    let (order_entry_tx, order_entry_rx): (Sender<Order>, Receiver<Order>) = mpsc::channel();

    let core_ids = core_affinity::get_core_ids().unwrap().into_iter().collect::<Vec<_>>();
    let pinned_match_core = core_ids[0];
    let pinned_msg_in_core = core_ids[1];
    let pinned_msg_out_core = core_ids[2];

    // OE and Match Thread
    let engine_thread = thread::spawn(move || {
        core_affinity::set_for_current(pinned_match_core);
        initialize_match_thread(engine_msg_out_tx, order_entry_rx);
    });

    wait_50_milli();

    // MULTICAST -> ENGINE MSG_IN
    let engine_msg_in_thread = thread::spawn(move || {
        core_affinity::set_for_current(pinned_msg_in_core);
        initialize_engine_msg_in_thread(order_entry_tx);
    });

    wait_50_milli();

    // ENGINE MSG_OUT -> MULTICAST
    let engine_msg_out_thread = thread::spawn(move || {
        core_affinity::set_for_current(pinned_msg_out_core);
        initialize_engine_msg_out_thread(engine_msg_out_rx)
    });

    engine_thread.join().unwrap();
    engine_msg_out_thread.join().unwrap();
    engine_msg_in_thread.join().unwrap();

    Ok(())
}