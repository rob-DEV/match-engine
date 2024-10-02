use crate::domain::execution::Execution;
use crate::domain::order::Order;
use crate::engine::order_book::CentralLimitOrderBook;
use crate::util::memory::uninitialized_arr;
use crate::util::time::epoch_nanos;
use common::engine::{CancelOrderAck, NewOrderAck, OutboundEngineMessage, OutboundMessage, TradeExecution};
use rand::random;
use std::sync::mpsc::Sender;
use std::{sync::{mpsc::Receiver, Arc, Mutex}, thread};

const MAX_EXECUTIONS_PER_CYCLE: usize = 2000;

pub struct MatchEngine {
    book_mutex: Arc<Mutex<CentralLimitOrderBook>>,
}

impl MatchEngine {
    pub fn new() -> MatchEngine {
        MatchEngine {
            book_mutex: Arc::new(Mutex::new(CentralLimitOrderBook::new()))
        }
    }

    pub fn run(&self, order_rx: Receiver<Order>, engine_msg_out_tx: Sender<OutboundEngineMessage>) {
        let book_mutex: Arc<Mutex<CentralLimitOrderBook>> = self.book_mutex.clone();

        let engine_msg_out_order_entry_tx = engine_msg_out_tx.clone();
        let _order_submission_thread_handle = thread::Builder::new()
            .name("ORDER-ENTRY-THREAD".to_owned())
            .spawn(move || Self::order_entry(book_mutex, order_rx, engine_msg_out_order_entry_tx));

        let book_handle_cycle_thread = self.book_mutex.clone();

        let engine_msg_out_match_cycle_tx = engine_msg_out_tx.clone();
        let _match_thread_handle = thread::Builder::new()
            .name("MATCH-CYCLE-THREAD".to_owned())
            .spawn(move || Self::matching_cycle(book_handle_cycle_thread, engine_msg_out_match_cycle_tx));
    }

    fn order_entry(book_mutex: Arc<Mutex<CentralLimitOrderBook>>, order_tx: Receiver<Order>, engine_msg_out_order_entry_tx: Sender<OutboundEngineMessage>) {
        while let Ok(order) = order_tx.recv() {
            let mut book = book_mutex.lock().unwrap();
            let out = match order {
                Order::New(new_order) => {
                    book.apply_order(new_order);
                    OutboundEngineMessage {
                        seq_num: 1,
                        outbound_message: OutboundMessage::NewOrderAck(NewOrderAck {
                            client_id: new_order.client_id,
                            action: new_order.action,
                            order_id: random::<u32>(),
                            px: new_order.px,
                            qty: new_order.qty,
                            ack_time: epoch_nanos(),
                        }),
                    }
                }
                Order::Cancel(cancel_order) => {
                    book.remove_order(cancel_order);
                    OutboundEngineMessage {
                        seq_num: 1,
                        outbound_message: OutboundMessage::CancelOrderAck(CancelOrderAck {
                            client_id: cancel_order.client_id,
                            order_id: random::<u32>(),
                            ack_time: epoch_nanos(),
                        }),
                    }
                }
            };

            engine_msg_out_order_entry_tx.send(out).unwrap()
        }
    }

    fn matching_cycle(book_handle: Arc<Mutex<CentralLimitOrderBook>>, engine_msg_out_tx: Sender<OutboundEngineMessage>) -> ! {
        let mut executions_buf = uninitialized_arr::<Execution, MAX_EXECUTIONS_PER_CYCLE>();

        let mut execution_seq_num = 0;

        loop {
            let mut book = book_handle.lock().unwrap();
            let executions = book.check_for_trades(MAX_EXECUTIONS_PER_CYCLE, &mut executions_buf);

            for index in 0..executions {
                let execution = &executions_buf[index];

                let outbound_execution_message;
                match execution {
                    Execution::FullMatch(full_match) => {
                        outbound_execution_message = OutboundEngineMessage {
                            seq_num: execution_seq_num,
                            outbound_message: OutboundMessage::TradeExecution(TradeExecution {
                                execution_id: full_match.id,
                                bid_client_id: full_match.bid.client_id,
                                bid_id: full_match.bid.id,
                                ask_client_id: full_match.ask.client_id,
                                ask_id: full_match.ask.id,
                                fill_qty: full_match.bid.qty,
                                px: full_match.bid.px,
                                execution_time: full_match.execution_time,
                            }),
                        }
                    }
                    Execution::PartialMatch(partial_match) => {
                        outbound_execution_message = OutboundEngineMessage {
                            seq_num: execution_seq_num,
                            outbound_message: OutboundMessage::TradeExecution(TradeExecution {
                                execution_id: partial_match.id,
                                bid_client_id: partial_match.bid.client_id,
                                bid_id: partial_match.bid.id,
                                ask_client_id: partial_match.ask.client_id,
                                ask_id: partial_match.ask.id,
                                fill_qty: partial_match.fill_qty,
                                px: partial_match.bid.px,
                                execution_time: partial_match.execution_time,
                            }),
                        }
                    }
                }

                engine_msg_out_tx.send(outbound_execution_message).unwrap();

                execution_seq_num += 1;
            }
        }
    }
}