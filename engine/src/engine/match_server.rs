use crate::domain::order::Order;
use crate::engine::engine_config::EngineConfig;
use crate::process::match_thread::match_thread;
use crate::process::msg_in_thread::msg_in_thread;
use crate::process::msg_out_thread::msg_out_thread;
use common::transport::sequenced_message::EngineMessage;
use common::util::time::wait_50_milli;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread::JoinHandle;

pub struct MatchServer {
    engine_config: EngineConfig,
    shutdown: AtomicBool,

    msg_in_thread: Option<JoinHandle<()>>,
    msg_out_thread: Option<JoinHandle<()>>,
    match_thread: Option<JoinHandle<()>>,
}

impl MatchServer {
    pub fn new(engine_config: EngineConfig) -> MatchServer {
        println!("Engine config:\n{:?}", engine_config);

        let (engine_msg_out_tx, engine_msg_out_rx): (
            Sender<EngineMessage>,
            Receiver<EngineMessage>,
        ) = mpsc::channel();
        let (order_entry_tx, order_entry_rx): (Sender<Order>, Receiver<Order>) = mpsc::channel();

        let core_ids = core_affinity::get_core_ids()
            .unwrap()
            .into_iter()
            .collect::<Vec<_>>();
        let pinned_match_core = core_ids[0];
        let pinned_msg_in_core = core_ids[1];
        let pinned_msg_out_core = core_ids[2];

        // OE and Match Thread
        let match_thread = match_thread(engine_msg_out_tx, order_entry_rx, pinned_match_core);

        wait_50_milli();

        // MULTICAST -> ENGINE MSG_IN
        let msg_in_thread = msg_in_thread(
            engine_config.msg_in_port,
            order_entry_tx,
            pinned_msg_in_core,
        );

        wait_50_milli();

        // ENGINE MSG_OUT -> MULTICAST
        let msg_out_thread = msg_out_thread(
            engine_config.msg_out_port,
            engine_msg_out_rx,
            pinned_msg_out_core,
        );

        MatchServer {
            engine_config,
            shutdown: AtomicBool::new(false),
            msg_in_thread: Some(msg_in_thread),
            msg_out_thread: Some(msg_out_thread),
            match_thread: Some(match_thread),
        }
    }

    pub fn run(&mut self) {
        self.msg_in_thread.take().unwrap().join().unwrap();
        self.msg_out_thread.take().unwrap().join().unwrap();
        self.match_thread.take().unwrap().join().unwrap();
    }
}
