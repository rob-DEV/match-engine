use crate::algorithm::algo_utils::{
    best_prices_cross, build_fill_execution, build_self_match_prevention_execution,
    traders_will_self_match,
};
use crate::algorithm::match_strategy::MatchStrategy;
use crate::book::order_book::LimitOrderBook;
use crate::domain::order::LimitOrder;
use common::message::execution_report::ExecutionReport;
use common::message::side::Side;

pub struct FifoMatchStrategy;

impl MatchStrategy for FifoMatchStrategy {
    fn new() -> Self {
        FifoMatchStrategy {}
    }

    fn match_orders(
        &mut self,
        order_book: &mut LimitOrderBook,
        order: &mut LimitOrder,
        executions_buffer: &mut Vec<ExecutionReport>,
    ) -> usize {
        let (_book_side, opposite_book_side) = match order.side {
            Side::BUY => (&mut order_book.bids, &mut order_book.asks),
            Side::SELL => (&mut order_book.asks, &mut order_book.bids),
        };

        loop {
            let Some(best_px) = opposite_book_side.best_price() else {
                break; // No liquidity - skip match, add to orderbook
            };

            if best_prices_cross(order, best_px) {
                let opposite_price_level =
                    match opposite_book_side.price_level_map.get_mut(&best_px) {
                        Some(l) => l,
                        None => break,
                    };

                // Match loop
                while order.qty > 0 && !opposite_price_level.is_empty() {
                    let resting_id = opposite_price_level.front().copied().unwrap();

                    let resting_order = match opposite_book_side.order_map.get_mut(&resting_id) {
                        Some(o) => o,
                        None => {
                            opposite_price_level.pop_front(); // cleanup orphan id
                            continue;
                        }
                    };

                    //SMP
                    if traders_will_self_match(order, resting_order) {
                        executions_buffer
                            .push(build_self_match_prevention_execution(resting_order));

                        opposite_price_level.pop_front();
                        continue;
                    }

                    let exec_qty = order.qty.min(resting_order.qty);
                    if exec_qty == 0 {
                        opposite_price_level.pop_front();
                        opposite_book_side.order_map.remove(&resting_id);
                        continue;
                    }

                    // Record execution
                    executions_buffer.push(build_fill_execution(
                        order,
                        resting_order,
                        best_px,
                        exec_qty,
                    ));

                    // Adjust quantities
                    // This needs abstracted
                    order.qty -= exec_qty;
                    resting_order.qty -= exec_qty;
                    opposite_price_level.total_qty -= exec_qty;
                    opposite_book_side.total_qty -= exec_qty;

                    // Remove fully filled book order
                    if resting_order.qty == 0 {
                        opposite_book_side.order_map.remove(&resting_id);
                        opposite_price_level.pop_front();
                        opposite_book_side.num_orders -= 1;
                    }
                }

                // Remove empty price level
                if opposite_price_level.is_empty() {
                    opposite_book_side.price_level_map.remove(&best_px);
                }

                if order.qty == 0 {
                    break;
                }
            } else {
                break;
            }
        }

        executions_buffer.len()
    }
}
