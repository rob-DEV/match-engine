use crate::book::book::Book;
use crate::book::fifo::opt_limit_order_book::OptLimitOrderBook;
use common::domain::execution::Execution;
use common::domain::order::Order;
use common::memory::memory::uninitialized_arr;
use common::domain::domain::{CancelOrderAck, NewOrderAck, TradeExecution};
use common::domain::messaging::{EngineMessage, SequencedEngineMessage};
use common::util::time::epoch_nanos;
use std::sync::mpsc::Sender;
use std::sync::mpsc::{Receiver, TryRecvError};

pub const MAX_EXECUTIONS_PER_CYCLE: usize = 2000;

pub struct MatchEngine {
    symbol: String,
    isin: String,
    book: OptLimitOrderBook,
}

impl MatchEngine {
    pub fn new(symbol: String, isin: String) -> Self {
        let mut book = OptLimitOrderBook::new();

        println!("--- Initializing engine instance for {symbol} (ISIN:{isin}) ---");
        Self {
            symbol,
            isin,
            book,
        }
    }

    pub fn run(&mut self, order_tx: Receiver<Order>, engine_msg_out_tx: Sender<SequencedEngineMessage>) -> ! {
        let order_cycle_msg_out_tx = engine_msg_out_tx.clone();
        let match_cycle_msg_out_tx = engine_msg_out_tx.clone();

        let mut engine_msg_out_seq_num: u32 = 1;

        let mut initial_order_seq_num: u32 = 1;
        let mut initial_execution_seq_num: u32 = 1;

        let mut timer_epoch = epoch_nanos();
        let mut order_seq_num = 1;
        let mut execution_sequence_number = 1;

        loop {
            let cycle_start_epoch = epoch_nanos();

            // oe phase
            (engine_msg_out_seq_num, initial_order_seq_num) = self.order_entry_cycle(engine_msg_out_seq_num, initial_order_seq_num, &order_tx, &order_cycle_msg_out_tx);

            // match phase
            (engine_msg_out_seq_num, initial_execution_seq_num) = self.match_cycle(engine_msg_out_seq_num, initial_execution_seq_num, &match_cycle_msg_out_tx);

            if epoch_nanos() - timer_epoch > 1000 * 1000 * 1000 {
                let nanos = epoch_nanos();
                println!("nanos: {} ord: {} exe: {} book: {}", nanos - cycle_start_epoch, initial_order_seq_num - order_seq_num, initial_execution_seq_num - execution_sequence_number, self.book.count_resting_orders());
                timer_epoch = nanos;
                order_seq_num = initial_order_seq_num;
                execution_sequence_number = initial_execution_seq_num;
            }
        }
    }

    fn order_entry_cycle(&mut self, mut engine_msg_out_seq_num: u32, mut order_seq_num: u32, order_tx: &Receiver<Order>, engine_msg_out_tx: &Sender<SequencedEngineMessage>) -> (u32, u32) {
        let order_result = order_tx.try_recv();
        match order_result {
            Ok(order) => {
                let mut book = &mut self.book;
                let out = match order {
                    Order::New(new_order) => {
                        book.apply(new_order);
                        SequencedEngineMessage {
                            sequence_number: engine_msg_out_seq_num,
                            message: EngineMessage::NewOrderAck(NewOrderAck {
                                client_id: new_order.client_id,
                                action: new_order.action,
                                order_id: new_order.id,
                                px: new_order.px,
                                qty: new_order.qty,
                                ack_time: epoch_nanos(),
                            }),
                        }
                    }
                    Order::Cancel(cancel_order) => {
                        let found = book.cancel(cancel_order);
                        SequencedEngineMessage {
                            sequence_number: engine_msg_out_seq_num,
                            message: EngineMessage::CancelOrderAck(CancelOrderAck {
                                client_id: cancel_order.client_id,
                                order_id: cancel_order.id,
                                found,
                                ack_time: epoch_nanos(),
                            }),
                        }
                    }
                };
                engine_msg_out_tx.send(out).unwrap();

                engine_msg_out_seq_num += 1;
                order_seq_num += 1;
            }
            Err(err) => {
                match err {
                    TryRecvError::Disconnected => { panic!("Error order recv disconnected!") }
                    _ => {}
                }
            }
        }
        return (engine_msg_out_seq_num, order_seq_num);
    }

    fn match_cycle(&mut self, mut engine_msg_out_seq_num: u32, mut execution_seq_num: u32, engine_msg_out_tx: &Sender<SequencedEngineMessage>) -> (u32, u32) {
        let mut executions_buf = uninitialized_arr::<Execution, MAX_EXECUTIONS_PER_CYCLE>();

        let num_executions = self.book.check_for_trades(MAX_EXECUTIONS_PER_CYCLE, &mut executions_buf);

        for idx in 0..num_executions {
            let execution = &executions_buf[idx];

            let outbound_execution_message;
            outbound_execution_message = SequencedEngineMessage {
                sequence_number: engine_msg_out_seq_num,
                message: EngineMessage::TradeExecution(TradeExecution {
                    trade_seq: execution_seq_num,
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

            engine_msg_out_seq_num += 1;
            execution_seq_num += 1;
        }

        return (engine_msg_out_seq_num, execution_seq_num);
    }
}