use crate::algorithm::fifo_match_strategy::FifoMatchStrategy;
use crate::algorithm::match_strategy::MatchStrategy;
use crate::book::book::Book;
use crate::book::order_book::LimitOrderBook;
use crate::domain::execution::Execution;
use crate::domain::order::{LimitOrder, Order};
use common::message::cancel_order::CancelOrderAck;
use common::message::execution::TradeExecution;
use common::message::instrument::Instrument;
use common::message::new_order::NewOrderAck;
use common::transport::sequenced_message::{EngineMessage, SequencedEngineMessage};
use common::util::time::system_nanos;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::{Sender, TryRecvError};

pub struct MatchEngine {
    instrument: Instrument,
    book: LimitOrderBook,
    match_strategy: FifoMatchStrategy,
    cycle_executions_buffer: Vec<Execution>,
}

impl MatchEngine {
    pub fn new(instrument: Instrument) -> Self {
        let book = LimitOrderBook::new();

        println!(
            "--- Initializing engine instance for {} ({}) ---",
            instrument.symbol, instrument.isin
        );
        Self {
            instrument,
            book,
            match_strategy: FifoMatchStrategy::new(),
            cycle_executions_buffer: Vec::with_capacity(100_000),
        }
    }

    pub fn run(
        &mut self,
        order_tx: Receiver<Order>,
        engine_msg_out_tx: Sender<SequencedEngineMessage>,
    ) -> ! {
        let match_cycle_msg_out_tx = engine_msg_out_tx.clone();

        let mut engine_msg_out_seq_num: u32 = 1;
        let mut engine_execution_seq_num = 1;

        // Per second statistics
        let mut statistic_print_per_second_clock = system_nanos();
        let mut orders_per_second = 0;
        let mut executions_per_second = 0;
        let mut cycles_per_second = 0;

        loop {
            let cycle_start_epoch_statistic = system_nanos();

            // oe phase
            if let Some(inbound_order) = self.receive_inbound_order(&order_tx) {
                match inbound_order {
                    Order::LimitOrder(mut limit_order) => {
                        orders_per_second += 1;

                        // match phase
                        let executions = self.match_inbound_order(
                            &mut engine_msg_out_seq_num,
                            &mut engine_execution_seq_num,
                            &mut limit_order,
                            &match_cycle_msg_out_tx,
                        );

                        if executions == 0 {
                            // ack full unmatched resting order - partial fill is implicitly resting
                            let out = SequencedEngineMessage {
                                sequence_number: engine_msg_out_seq_num,
                                message: EngineMessage::NewOrderAck(NewOrderAck {
                                    client_id: limit_order.client_id,
                                    side: limit_order.side,
                                    order_id: limit_order.id,
                                    px: limit_order.px,
                                    qty: limit_order.qty,
                                    ack_time: system_nanos(),
                                }),
                                sent_time: system_nanos(),
                            };
                            engine_msg_out_tx.send(out).unwrap();
                            engine_msg_out_seq_num += 1;
                        } else {
                            executions_per_second += executions;
                        }
                    }
                    Order::Cancel(cancel_order) => {
                        let found = self.book.remove_order(&cancel_order);
                        let out = SequencedEngineMessage {
                            sequence_number: engine_msg_out_seq_num,
                            message: EngineMessage::CancelOrderAck(CancelOrderAck {
                                client_id: cancel_order.client_id,
                                order_id: cancel_order.order_id,
                                found,
                                ack_time: system_nanos(),
                            }),
                            sent_time: system_nanos(),
                        };
                        engine_msg_out_tx.send(out).unwrap();
                        engine_msg_out_seq_num += 1;
                        orders_per_second += 1;
                    }
                }
            }

            cycles_per_second += 1;

            if system_nanos() - statistic_print_per_second_clock > 1000 * 1000 * 1000 {
                let nanos = system_nanos();
                println!(
                    "nanos: {} ord: {} exe: {} book: {} bid_v: {} ask_v: {} volume: {}",
                    nanos - cycle_start_epoch_statistic,
                    // cycles_per_second,
                    orders_per_second,
                    executions_per_second,
                    self.book.orders_on_book(),
                    self.book.bid_volume(),
                    self.book.ask_volume(),
                    self.book.total_volume()
                );

                statistic_print_per_second_clock = nanos;
                orders_per_second = 0;
                executions_per_second = 0;
                cycles_per_second = 0;
            }
        }
    }

    fn receive_inbound_order(&mut self, order_tx: &Receiver<Order>) -> Option<Order> {
        let inbound_order = order_tx.try_recv();

        match inbound_order {
            Ok(order) => Some(order),
            Err(err) => match err {
                TryRecvError::Disconnected => {
                    panic!("Error order recv disconnected!")
                }
                _ => None,
            },
        }
    }

    fn match_inbound_order(
        &mut self,
        engine_msg_out_seq_num: &mut u32,
        execution_seq_num: &mut u32,
        order: &mut LimitOrder,
        engine_msg_out_tx: &Sender<SequencedEngineMessage>,
    ) -> u32 {
        self.cycle_executions_buffer.clear();

        let num_executions = self.match_strategy.match_orders(
            &mut self.book,
            order,
            &mut self.cycle_executions_buffer,
        );

        self.cycle_executions_buffer
            .iter()
            .for_each(|execution: &Execution| {
                let outbound_execution_message;
                outbound_execution_message = SequencedEngineMessage {
                    sequence_number: *engine_msg_out_seq_num,
                    message: EngineMessage::TradeExecution(TradeExecution {
                        trade_seq: *execution_seq_num,
                        trade_id: execution.id,
                        bid_client_id: execution.bid.client_id,
                        ask_client_id: execution.ask.client_id,
                        bid_order_id: execution.bid.id,
                        ask_order_id: execution.ask.id,
                        exec_qty: execution.exec_qty,
                        exec_type: execution.exec_type,
                        px: execution.bid.px,
                        execution_time: execution.execution_time,
                    }),
                    sent_time: system_nanos(),
                };

                engine_msg_out_tx.send(outbound_execution_message).unwrap();

                *engine_msg_out_seq_num += 1;
                *execution_seq_num += 1;
            });

        num_executions as u32
    }
}
