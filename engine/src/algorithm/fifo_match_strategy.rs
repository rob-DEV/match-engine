use crate::book::book::Book;
use common::domain::domain::Side;
use common::domain::execution::Execution;
use common::domain::order::LimitOrder;
use common::util::time::epoch_nanos;
use rand::random;
use std::cmp::Ordering;

use crate::algorithm::match_strategy::MatchStrategy;
use crate::book::opt_limit_order_book::OptLimitOrderBook;

pub struct FifoMatch;

impl MatchStrategy for FifoMatch {
    fn match_orders(
        &self,
        order_book: &mut OptLimitOrderBook,
        mutable_execution_buffer: &mut [Execution],
    ) -> usize {
        let mut num_executions: usize = 0;

        while let (Some(mut ask_order_list_entry), Some(mut bid_order_list_entry)) = (
            order_book.asks.price_tree_map.first_entry(),
            order_book.bids.price_tree_map.last_entry(),
        ) {
            let ask_price = ask_order_list_entry.key();
            let bid_price = bid_order_list_entry.key();

            if ask_price > bid_price {
                return num_executions;
            }

            let order_result: Vec<(u32, u32, Option<LimitOrder>)> = ask_order_list_entry
                .get()
                .iter()
                .zip(bid_order_list_entry.get().iter())
                .map(|(ask, bid)| {
                    let (ask, bid) = match (ask.action, bid.action) {
                        (Side::BUY, Side::SELL) => (bid, ask),
                        (Side::SELL, Side::BUY) => (ask, bid),
                        (_, _) => return None,
                    };

                    if ask.px > bid.px {
                        return None;
                    }

                    let execution = match ask.qty.cmp(&bid.qty) {
                        Ordering::Equal => Some((
                            Execution {
                                id: random::<u32>(),
                                fill_qty: ask.qty,
                                ask: ask.clone(),
                                bid: bid.clone(),
                                execution_time: epoch_nanos(),
                            },
                            None,
                        )),
                        Ordering::Greater => {
                            let quantity = bid.qty;
                            let mut remainder = ask.clone();
                            remainder.qty -= quantity;

                            Some((
                                Execution {
                                    id: random::<u32>(),
                                    fill_qty: quantity,
                                    ask: ask.clone(),
                                    bid: bid.clone(),
                                    execution_time: epoch_nanos(),
                                },
                                Some(remainder),
                            ))
                        }
                        Ordering::Less => {
                            let quantity = ask.qty;
                            let mut remainder = bid.clone();
                            remainder.qty -= quantity;
                            Some((
                                Execution {
                                    id: random::<u32>(),
                                    fill_qty: quantity,
                                    ask: ask.clone(),
                                    bid: bid.clone(),
                                    execution_time: epoch_nanos(),
                                },
                                Some(remainder),
                            ))
                        }
                    };

                    match execution {
                        None => None,
                        Some((execution, rem)) => {
                            mutable_execution_buffer[num_executions] = execution;
                            num_executions += 1;
                            Some((ask.id, bid.id, rem))
                        }
                    }
                })
                .filter(|x| x.is_some())
                .map(|x| x.unwrap())
                .collect::<Vec<_>>();

            ask_order_list_entry.get_mut().pop_front();
            bid_order_list_entry.get_mut().pop_back();

            order_result.iter().for_each(|(x, t, v)| {
                order_book.asks.remove_order(*x);
                order_book.bids.remove_order(*t);

                match v {
                    None => {}
                    Some(a) => match a.action {
                        Side::BUY => order_book.bids.add_order(*a),
                        Side::SELL => order_book.asks.add_order(*a),
                    },
                }
            })
        }

        num_executions
    }
}
