use crate::algorithm::fifo_match_strategy::FifoMatchStrategy;
use crate::algorithm::match_strategy::MatchStrategy;
use crate::book::book::Book;
use crate::book::order_book::LimitOrderBook;
use crate::domain::order::{LimitOrder, Order};
use common::transport::sequenced_message::EngineMessage;
use common::types::cancel_order::Reason::ClientRequested;
use common::types::cancel_order::{CancelOrderStatus, CancelledOrderAck};
use common::types::execution_report::ExecutionReport;
use common::types::order::NewOrderAck;
use common::util::time::system_nanos;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::{Sender, TryRecvError};

pub struct MatchEngine {
    book: LimitOrderBook,
    match_strategy: FifoMatchStrategy,
    cycle_executions_buffer: Vec<ExecutionReport>,
}

impl MatchEngine {
    pub fn new() -> Self {
        let book = LimitOrderBook::new();

        println!("--- Initializing engine instance ---",);
        Self {
            book,
            match_strategy: FifoMatchStrategy::new(),
            cycle_executions_buffer: Vec::with_capacity(100_000),
        }
    }

    pub fn run(
        &mut self,
        order_tx: Receiver<Order>,
        engine_msg_out_tx: Sender<EngineMessage>,
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

                        // add & ack full / remainder order
                        if limit_order.qty > 0 {
                            self.book.add_order(limit_order);
                            let out = EngineMessage::NewOrderAck(NewOrderAck {
                                client_id: limit_order.client_id,
                                side: limit_order.side,
                                order_id: limit_order.id,
                                px: limit_order.px,
                                qty: limit_order.qty,
                                ack_time: system_nanos(),
                            });

                            engine_msg_out_tx.send(out).unwrap();
                            engine_msg_out_seq_num += 1;
                        }

                        executions_per_second += executions;
                    }
                    Order::Cancel(cancel_order) => {
                        let cancel_order_status = match self.book.remove_order(&cancel_order) {
                            true => CancelOrderStatus::Cancelled,
                            false => CancelOrderStatus::NotFound,
                        };

                        let out = EngineMessage::CancelOrderAck(CancelledOrderAck {
                            client_id: cancel_order.client_id,
                            order_id: cancel_order.order_id,
                            instrument: [0; 16],
                            cancel_order_status,
                            reason: ClientRequested,
                            ack_time: system_nanos(),
                        });

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
        engine_msg_out_tx: &Sender<EngineMessage>,
    ) -> u32 {
        self.cycle_executions_buffer.clear();

        let num_executions = self.match_strategy.match_orders(
            &mut self.book,
            order,
            &mut self.cycle_executions_buffer,
        );

        for execution_report in self.cycle_executions_buffer.drain(..) {
            let outbound_execution_message = EngineMessage::TradeExecution(execution_report);

            engine_msg_out_tx.send(outbound_execution_message).unwrap();

            *engine_msg_out_seq_num += 1;
            *execution_seq_num += 1;
        }

        num_executions as u32
    }
}
