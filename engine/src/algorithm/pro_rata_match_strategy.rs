use crate::algorithm::match_strategy::MatchStrategy;
use crate::book::book::Book;
use crate::book::opt_limit_order_book::{LimitOrderList, OptLimitOrderBook, Price};
use crate::engine::match_engine::MAX_EXECUTIONS_PER_CYCLE;
use common::domain::domain::Side::{BUY, SELL};
use common::domain::execution::Execution;
use common::domain::order::CancelOrder;
use common::util::time::epoch_nanos;
use rand::random;
use std::collections::btree_map::OccupiedEntry;

pub struct ProRataMatchStrategy;

impl MatchStrategy for ProRataMatchStrategy {
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
            let bid_price = bid_order_list_entry.key();
            let ask_price = ask_order_list_entry.key();

            if *ask_price > *bid_price {
                return num_executions;
            }

            let total_bid_qty_at_price: u32 = bid_order_list_entry
                .get()
                .iter()
                .map(|order| order.qty)
                .sum();
            let total_ask_qty_at_price: u32 = ask_order_list_entry
                .get()
                .iter()
                .map(|order| order.qty)
                .sum();

            println!(
                "PriceLevel: {} Asks at level: {} Bids at level: {}",
                bid_price, total_bid_qty_at_price, total_ask_qty_at_price
            );

            let matched_qty = total_bid_qty_at_price.min(total_ask_qty_at_price);

            if matched_qty == 0 {
                break;
            }

            let mut bid_allocs: Vec<u32> = Self::pro_rata_allocate_fills(
                &mut bid_order_list_entry,
                total_bid_qty_at_price,
                matched_qty,
            );
            let mut ask_allocs: Vec<u32> = Self::pro_rata_allocate_fills(
                &mut ask_order_list_entry,
                total_ask_qty_at_price,
                matched_qty,
            );

            if num_executions + ask_allocs.len() * (bid_allocs.len()) >= MAX_EXECUTIONS_PER_CYCLE {
                println!("Overflowing at {}", num_executions);
                return num_executions;
            }

            Self::check_for_allocation_rounding_errors(
                &mut bid_order_list_entry,
                total_bid_qty_at_price,
                matched_qty,
                &mut bid_allocs,
            );
            Self::check_for_allocation_rounding_errors(
                &mut ask_order_list_entry,
                total_ask_qty_at_price,
                matched_qty,
                &mut ask_allocs,
            );

            // at this point, all available qty should be allocated with any qty missed in integer rounding being given to the largest party

            assert_eq!(bid_allocs.iter().sum::<u32>(), matched_qty);
            assert_eq!(ask_allocs.iter().sum::<u32>(), matched_qty);

            // Phase 1: compute allocations
            let mut bid_iter = bid_order_list_entry.get().iter();
            let mut ask_iter = ask_order_list_entry.get().iter();

            let mut bid_idx = 0;
            let mut ask_idx = 0;

            let mut bids_to_cancel: Vec<u32> = Vec::new();
            let mut asks_to_cancel: Vec<u32> = Vec::new();
            let mut bids_to_modify: Vec<(u32, u32)> = Vec::new();
            let mut asks_to_modify: Vec<(u32, u32)> = Vec::new();

            while let (Some(bid_order), Some(ask_order)) = (bid_iter.next(), ask_iter.next()) {
                let fill_qty = bid_allocs[bid_idx].min(ask_allocs[ask_idx]);
                if fill_qty == 0 {
                    continue;
                }

                // push execution into buffer
                mutable_execution_buffer[num_executions] = Execution {
                    bid: bid_order.clone(),
                    ask: ask_order.clone(),
                    fill_qty,
                    id: random::<u32>(),
                    execution_time: epoch_nanos(),
                };

                bid_allocs[bid_idx] -= fill_qty;
                ask_allocs[ask_idx] -= fill_qty;

                if bid_allocs[bid_idx] == 0 {
                    bids_to_cancel.push(bid_order.id);
                } else {
                    bids_to_modify.push((bid_order.id, bid_allocs[bid_idx]));
                }

                if ask_allocs[ask_idx] == 0 {
                    asks_to_cancel.push(ask_order.id);
                } else {
                    asks_to_modify.push((ask_order.id, ask_allocs[ask_idx]));
                }

                if bid_allocs[bid_idx] == 0 {
                    bid_idx += 1;
                }
                if ask_allocs[ask_idx] == 0 {
                    ask_idx += 1;
                }
            }

            bids_to_modify
                .iter()
                .for_each(|(id, qty)| order_book.modify_order(BUY, *id, *qty));
            asks_to_modify
                .iter()
                .for_each(|(id, qty)| order_book.modify_order(SELL, *id, *qty));
            bids_to_cancel.iter().for_each(|id| {
                order_book.remove_order(CancelOrder {
                    client_id: 0,
                    action: BUY,
                    id: *id,
                });
            });
            asks_to_cancel.iter().for_each(|id| {
                order_book.remove_order(CancelOrder {
                    client_id: 0,
                    action: SELL,
                    id: *id,
                });
            });
        }

        num_executions
    }
}

impl ProRataMatchStrategy {
    fn pro_rata_allocate_fills(
        bid_order_list_entry: &mut OccupiedEntry<Price, LimitOrderList>,
        total_bid_qty_at_price: u32,
        matched_qty: u32,
    ) -> Vec<u32> {
        bid_order_list_entry
            .get()
            .iter()
            .map(|o| {
                (o.qty as f64 * matched_qty as f64 / total_bid_qty_at_price as f64).floor() as u32
            })
            .collect()
    }

    fn check_for_allocation_rounding_errors(
        order_list_entry: &mut OccupiedEntry<Price, LimitOrderList>,
        total_order_list_qty: u32,
        matched_qty: u32,
        pro_rata_allocation: &mut Vec<u32>,
    ) -> u32 {
        let leftover_unallocated_qty = matched_qty - pro_rata_allocation.iter().sum::<u32>();

        if leftover_unallocated_qty > 0 {
            let mut remainders: Vec<(usize, f64)> = order_list_entry
                .get()
                .iter()
                .enumerate()
                .map(|(i, o)| {
                    let frac = (o.qty as f64 * matched_qty as f64 / total_order_list_qty as f64)
                        - pro_rata_allocation[i] as f64;
                    (i, frac)
                })
                .collect();
            remainders.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
            for i in 0..leftover_unallocated_qty as usize {
                pro_rata_allocation[remainders[i].0] += 1;
            }
        }
        leftover_unallocated_qty
    }
}
