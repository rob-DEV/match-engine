use crate::algorithm::match_strategy::MatchStrategy;
use crate::book::book::Book;
use crate::book::order_book::{LimitOrderBook, Price};
use common::domain::domain::Side;
use common::domain::execution::Execution;
use common::domain::order::LimitOrder;
use common::util::time::epoch_nanos;
use rand::random;

pub struct FifoMatchStrategy;

impl MatchStrategy for FifoMatchStrategy {
    fn new() -> Self {
        FifoMatchStrategy {}
    }

    fn match_orders(
        &mut self,
        order_book: &mut LimitOrderBook,
        order: &mut LimitOrder,
        mutable_execution_buffer: &mut [Execution],
    ) -> usize {
        let mut num_executions = 0;

        let (book_side, opposite_book_side) = match order.side {
            Side::BUY => (&mut order_book.bids, &mut order_book.asks),
            Side::SELL => (&mut order_book.asks, &mut order_book.bids),
        };

        loop {
            let Some(best_px) = opposite_book_side.best_price() else {
                break; // No liquidity - skip match, add to orderbook
            };

            let px_cross = Self::best_prices_cross(order, best_px);

            if px_cross {
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

                    let fill_qty = order.qty.min(resting_order.qty);
                    if fill_qty == 0 {
                        opposite_price_level.pop_front();
                        opposite_book_side.order_map.remove(&resting_id);
                        continue;
                    }

                    // Adjust quantities
                    // This needs abstracted
                    order.qty -= fill_qty;
                    resting_order.qty -= fill_qty;
                    opposite_price_level.total_qty -= fill_qty;
                    opposite_book_side.total_qty -= fill_qty;

                    // Record execution
                    mutable_execution_buffer[num_executions] =
                        Self::build_fill_execution(order, resting_order, fill_qty);
                    num_executions += 1;

                    // Remove fully filled book order
                    if resting_order.qty == 0 {
                        opposite_book_side.order_map.remove(&resting_id);
                        opposite_price_level.pop_front();
                        opposite_book_side.num_orders -= 1;
                    }

                    if num_executions >= mutable_execution_buffer.len() {
                        return num_executions;
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

        // If still unfilled, add to book
        if order.qty > 0 {
            order_book.add_order(*order)
        }

        num_executions
    }
}

impl FifoMatchStrategy {
    fn best_prices_cross(order: &mut LimitOrder, best_px: Price) -> bool {
        let px_cross = match order.side {
            Side::BUY => order.px >= best_px,
            Side::SELL => order.px <= best_px,
        };
        px_cross
    }

    fn build_fill_execution(
        order: &mut LimitOrder,
        resting_order: &mut LimitOrder,
        fill_qty: u32,
    ) -> Execution {
        let bid = match order.side {
            Side::BUY => order.clone(),
            Side::SELL => resting_order.clone(),
        };

        let ask = match order.side {
            Side::BUY => resting_order.clone(),
            Side::SELL => order.clone(),
        };

        Execution {
            id: random::<u32>(),
            fill_qty,
            bid,
            ask,
            execution_time: epoch_nanos(),
        }
    }
}
