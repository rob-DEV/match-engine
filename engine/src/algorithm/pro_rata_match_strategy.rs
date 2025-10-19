use crate::algorithm::match_strategy::MatchStrategy;
use crate::book::book::Book;
use crate::book::opt_limit_order_book::{LimitOrderList, OptLimitOrderBook, Price};
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
       
        num_executions
    }
}

impl ProRataMatchStrategy {
    fn pro_rata_allocate_fills_remainder_to_largest_party(
        order_list_entry: &mut OccupiedEntry<Price, LimitOrderList>,
        total_qty_at_price: u32,
        matched_qty: u32,
    ) -> Vec<(u32, u32, u32)> {
        let mut largest_allocation = 0;
        let mut max_allocation_idx = 0;

        let mut pro_rata_allocations: Vec<(u32, u32, u32)> = order_list_entry
            .get()
            .iter()
            .enumerate()
            .map(|(idx, o)| {
                let allocation =
                    (o.qty as f64 * matched_qty as f64 / total_qty_at_price as f64).floor() as u32;

                if allocation > largest_allocation {
                    println!("Max {}", allocation);
                    largest_allocation = allocation;
                    max_allocation_idx = idx;
                }

                return (o.id, o.qty, allocation);
            })
            .collect();

        let remainder_qty = matched_qty
            - pro_rata_allocations
                .iter()
                .map(|(_order_id, qty, allocation)| allocation)
                .sum::<u32>();

        if remainder_qty > 0 {
            pro_rata_allocations[max_allocation_idx].2 =
                pro_rata_allocations[max_allocation_idx].2 + remainder_qty;
        }

        pro_rata_allocations
    }

    fn calculate_total_qty_at_level(
        order_list_entry: &mut OccupiedEntry<Price, LimitOrderList>,
    ) -> u32 {
        order_list_entry.get().iter().map(|order| order.qty).sum()
    }
}
