use crate::algorithm::fifo_match_strategy::FifoMatchStrategy;
use crate::algorithm::match_strategy::MatchStrategy;
use crate::book::book::Book;
use crate::book::order_book::LimitOrderBook;
use common::domain::domain::{CancelOrderAck, NewOrderAck, TradeExecution};
use common::domain::execution::Execution;
use common::domain::messaging::{EngineMessage, SequencedEngineMessage};
use common::domain::order::{LimitOrder, Order};
use common::memory::memory::uninitialized_arr;
use common::util::time::epoch_nanos;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::{Sender, TryRecvError};

pub const MAX_EXECUTIONS_PER_CYCLE: usize = 4096;

pub struct MatchEngine {
    symbol: String,
    isin: String,
    book: LimitOrderBook,
    match_strategy: FifoMatchStrategy,
}

impl MatchEngine {
    pub fn new(symbol: String, isin: String) -> Self {
        let book = LimitOrderBook::new();

        println!("--- Initializing engine instance for {symbol} (ISIN:{isin}) ---");
        Self {
            symbol,
            isin,
            book,
            match_strategy: FifoMatchStrategy::new(),
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
        let mut statistic_print_per_second_clock = epoch_nanos();
        let mut orders_per_second = 0;
        let mut executions_per_second = 0;

        loop {
            let cycle_start_epoch_statistic = epoch_nanos();

            // oe phase
            if let Some(inbound_order) = self.receive_inbound_order(&order_tx) {
                match inbound_order {
                    Order::New(mut limit_order) => {
                        let out = SequencedEngineMessage {
                            sequence_number: engine_msg_out_seq_num,
                            message: EngineMessage::NewOrderAck(NewOrderAck {
                                client_id: limit_order.client_id,
                                side: limit_order.side,
                                order_id: limit_order.id,
                                px: limit_order.px,
                                qty: limit_order.qty,
                                ack_time: epoch_nanos(),
                            }),
                        };
                        engine_msg_out_tx.send(out).unwrap();
                        engine_msg_out_seq_num += 1;
                        orders_per_second += 1;

                        // match phase
                        let executions = self.match_inbound_order(
                            &mut engine_msg_out_seq_num,
                            &mut engine_execution_seq_num,
                            &mut limit_order,
                            &match_cycle_msg_out_tx,
                        );

                        executions_per_second += executions;
                    }
                    Order::Cancel(cancel_order) => {
                        let found = self.book.remove_order(cancel_order);
                        let out = SequencedEngineMessage {
                            sequence_number: engine_msg_out_seq_num,
                            message: EngineMessage::CancelOrderAck(CancelOrderAck {
                                client_id: cancel_order.client_id,
                                order_id: cancel_order.id,
                                found,
                                ack_time: epoch_nanos(),
                            }),
                        };
                        engine_msg_out_tx.send(out).unwrap();
                        engine_msg_out_seq_num += 1;
                        orders_per_second += 1;
                    }
                }
            }

            if epoch_nanos() - statistic_print_per_second_clock > 1000 * 1000 * 1000 {
                let nanos = epoch_nanos();
                println!(
                    "nanos: {} ord: {} exe: {} book: {} bid_v: {} ask_v: {} volume: {}",
                    nanos - cycle_start_epoch_statistic,
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
        let mut executions_buf = uninitialized_arr::<Execution, MAX_EXECUTIONS_PER_CYCLE>();

        let num_executions =
            self.match_strategy
                .match_orders(&mut self.book, order, &mut executions_buf);

        for idx in 0..num_executions {
            let execution = &executions_buf[idx];

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
                    fill_qty: execution.fill_qty,
                    px: execution.bid.px,
                    execution_time: execution.execution_time,
                }),
            };

            engine_msg_out_tx.send(outbound_execution_message).unwrap();

            *engine_msg_out_seq_num += 1;
            *execution_seq_num += 1;
        }

        num_executions as u32
    }
}
