use crate::domain::execution::Execution;
use crate::domain::order::Order;
use crate::engine::order_book::LimitOrderBook;
use crate::util::memory::uninitialized_arr;
use crate::util::time::epoch_nanos;
use common::engine::{CancelOrderAck, NewOrderAck, OutboundEngineMessage, OutboundMessage, TradeExecution};
use std::sync::mpsc::Sender;
use std::sync::mpsc::{Receiver, TryRecvError};

const MAX_ORDERS_PER_CYCLE: u32 = 2000;
const MAX_EXECUTIONS_PER_CYCLE: usize = 2000;
const MAX_ORDER_CYCLE_NANOS: u64 = 10000;

pub struct MatchEngine {
    book: LimitOrderBook,
}

impl MatchEngine {
    pub fn new() -> MatchEngine {
        MatchEngine {
            book: LimitOrderBook::new()
        }
    }

    pub(crate) fn run(&mut self, order_tx: Receiver<Order>, engine_msg_out_tx: Sender<OutboundEngineMessage>) -> ! {
        let order_cycle_msg_out_tx = engine_msg_out_tx.clone();
        let match_cycle_msg_out_tx = engine_msg_out_tx.clone();

        let mut engine_msg_out_seq_num: u32 = 1;

        let mut order_seq_num: u32 = 1;
        let mut execution_seq_num: u32 = 1;

        loop {
            let cycle_start_epoch = epoch_nanos();

            // order entry
            (engine_msg_out_seq_num, order_seq_num) = self.order_entry_cycle(engine_msg_out_seq_num, order_seq_num, cycle_start_epoch, &order_tx, &order_cycle_msg_out_tx);

            // execution phase
            (engine_msg_out_seq_num, execution_seq_num) = self.match_cycle(engine_msg_out_seq_num, execution_seq_num, &match_cycle_msg_out_tx);
        }
    }

    fn order_entry_cycle(&mut self, mut engine_msg_out_seq_num: u32, mut order_sequence_num: u32, cycle_start_epoch: u64, order_tx: &Receiver<Order>, engine_msg_out_tx: &Sender<OutboundEngineMessage>) -> (u32, u32) {
        let initial_order_seq_number: u32 = order_sequence_num;

        while order_sequence_num - initial_order_seq_number < MAX_ORDERS_PER_CYCLE && epoch_nanos() - cycle_start_epoch < MAX_ORDER_CYCLE_NANOS {
            let order_result = order_tx.try_recv();
            match order_result {
                Ok(order) => {
                    let mut book = &mut self.book;
                    let out = match order {
                        Order::New(new_order) => {
                            book.apply_order(new_order);
                            OutboundEngineMessage {
                                seq_num: engine_msg_out_seq_num,
                                outbound_message: OutboundMessage::NewOrderAck(NewOrderAck {
                                    client_id: new_order.client_id,
                                    action: new_order.action,
                                    order_id: order_sequence_num,
                                    px: new_order.px,
                                    qty: new_order.qty,
                                    ack_time: epoch_nanos(),
                                }),
                            }
                        }
                        Order::Cancel(cancel_order) => {
                            book.remove_order(cancel_order);
                            OutboundEngineMessage {
                                seq_num: engine_msg_out_seq_num,
                                outbound_message: OutboundMessage::CancelOrderAck(CancelOrderAck {
                                    client_id: cancel_order.client_id,
                                    order_id: order_sequence_num,
                                    ack_time: epoch_nanos(),
                                }),
                            }
                        }
                    };
                    engine_msg_out_tx.send(out).unwrap();

                    engine_msg_out_seq_num += 1;
                    order_sequence_num += 1;
                }
                Err(err) => {
                    match err {
                        TryRecvError::Disconnected => { panic!("Error order recv disconnected!") }
                        _ => {}
                    }
                }
            }
        }

        return (engine_msg_out_seq_num, order_sequence_num);
    }

    fn match_cycle(&mut self, mut engine_msg_out_seq_num: u32, mut execution_seq_num: u32, engine_msg_out_tx: &Sender<OutboundEngineMessage>) -> (u32, u32) {
        let mut executions_buf = uninitialized_arr::<Execution, MAX_EXECUTIONS_PER_CYCLE>();

        let mut book = &mut self.book;
        let executions = book.check_for_trades(MAX_EXECUTIONS_PER_CYCLE, &mut executions_buf);

        for idx in 0..executions {
            let execution = &executions_buf[idx];

            let outbound_execution_message;
            match execution {
                Execution::FullMatch(full_match) => {
                    outbound_execution_message = OutboundEngineMessage {
                        seq_num: engine_msg_out_seq_num,
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
                        seq_num: engine_msg_out_seq_num,
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

            engine_msg_out_seq_num += 1;
            execution_seq_num += 1;
        }

        return (engine_msg_out_seq_num, execution_seq_num);
    }
}