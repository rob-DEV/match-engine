use crate::algorithm::match_strategy::MatchStrategy;
use crate::book::book::Book;
use crate::book::opt_limit_order_book::{LimitOrderList, OptLimitOrderBook, Price};
use common::domain::domain::Side;
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

            if ask_price > bid_price {
                return num_executions;
            }

            // on price level x
            println!("On price level ask {} bid {}", ask_price, bid_price);
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
                "Asks at level: {} Bids at level: {}",
                total_bid_qty_at_price, total_ask_qty_at_price
            );

            let matched_qty = total_bid_qty_at_price.min(total_ask_qty_at_price);

            if matched_qty == 0 {
                break;
            }

            println!("Matching {} units at price {}", matched_qty, ask_price);

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

            println!("bid_allocs: {:?}", bid_allocs);
            println!("ask_allocs: {:?}", ask_allocs);

            println!("Checking for any rounding issues on allocation");

            let leftover_bids = Self::check_for_allocation_rounding_errors(
                &mut bid_order_list_entry,
                total_bid_qty_at_price,
                matched_qty,
                &mut bid_allocs,
            );
            println!("Leftover bids: {}", leftover_bids);
            let leftover_asks = Self::check_for_allocation_rounding_errors(
                &mut ask_order_list_entry,
                total_ask_qty_at_price,
                matched_qty,
                &mut ask_allocs,
            );
            println!("Leftover asks: {}", leftover_asks);

            // at this point, all available qty should be allocated with any qty missed in integer rounding being given to the largest party

            println!("Final allocations: {:?} \n {:?}", bid_allocs, ask_allocs);

            assert_eq!(bid_allocs.iter().sum::<u32>() + ask_allocs.iter().sum::<u32>(), matched_qty);

           
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
                // o.id,
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
