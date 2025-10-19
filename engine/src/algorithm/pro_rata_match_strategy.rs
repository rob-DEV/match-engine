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
        while let (Some(mut bid_order_list_entry), Some(mut ask_order_list_entry)) = (
            order_book.bids.price_tree_map.last_entry(),
            order_book.asks.price_tree_map.first_entry(),
        ) {
            let highest_bid_price = bid_order_list_entry.key();
            let lowest_ask_price = ask_order_list_entry.key();

            if (highest_bid_price < lowest_ask_price) {
                println!("No crossing of bids and asks");
                break;
            }

            let total_bid_qty_at_price: u32 =
                Self::calculate_total_qty_at_level(&mut bid_order_list_entry);
            let total_ask_qty_at_price: u32 =
                Self::calculate_total_qty_at_level(&mut ask_order_list_entry);

            let fillable_qty = total_bid_qty_at_price.min(total_ask_qty_at_price);

            let mut bid_allocs: Vec<(u32, u32, u32)> =
                Self::pro_rata_allocate_fills_remainder_to_largest_party(
                    &mut bid_order_list_entry,
                    total_bid_qty_at_price,
                    fillable_qty,
                );
            let mut ask_allocs: Vec<(u32, u32, u32)> =
                Self::pro_rata_allocate_fills_remainder_to_largest_party(
                    &mut ask_order_list_entry,
                    total_ask_qty_at_price,
                    fillable_qty,
                );
            
            println!(
                "Final allocations: \nbid_allocs: {:?}\nask_allocs: {:?}",
                bid_allocs, ask_allocs
            );
            // bid_allocs [10, 10]
            // ask_allocs [20]

            // exhaustively fill the allocs only moving to the next when 0
            // mark 0 orders to be removed
            // modify reduced orders
            let mut bid_alloc_iter = bid_allocs.iter();
            let mut ask_alloc_iter = ask_allocs.iter();

            let mut bid_book_iter = bid_order_list_entry.get().iter();
            let mut ask_book_iter = ask_order_list_entry.get().iter();

            let mut remaining_qty = fillable_qty;

            // needs fixed
            let mut current_bid_alloc = bid_alloc_iter.next().unwrap();
            let mut current_ask_alloc = ask_alloc_iter.next().unwrap();

            let mut current_bid_order = bid_book_iter.next().unwrap();
            let mut current_ask_order = ask_book_iter.next().unwrap();

            while remaining_qty > 0 {
                let (_, _, current_bid_allocation) = current_bid_alloc;
                let (_, _, current_ask_allocation) = current_ask_alloc;

                let current_fillable_qty = (*current_bid_allocation).min(*current_ask_allocation);

                let book_bid_order = current_bid_order;
                let book_ask_order = current_ask_order;

                let execution = Execution {
                    id: random::<u32>(),
                    ask: book_ask_order.clone(),
                    bid: book_bid_order.clone(),
                    fill_qty: current_fillable_qty,
                    execution_time: epoch_nanos(),
                };

                mutable_execution_buffer[num_executions] = execution;
                num_executions += 1;

                remaining_qty -= current_fillable_qty;

                if remaining_qty > 0 && *current_bid_allocation - current_fillable_qty == 0 {
                    println!("Advancing bid alloc iter to next item");
                    current_bid_alloc = bid_alloc_iter.next().unwrap();
                    current_bid_order = bid_book_iter.next().unwrap();
                }
                if remaining_qty > 0 && *current_ask_allocation - current_fillable_qty == 0 {
                    println!("Advancing ask alloc iter to next item");
                    current_ask_alloc = ask_alloc_iter.next().unwrap();
                    current_ask_order = ask_book_iter.next().unwrap();
                }
            }

            // cleanup
            bid_allocs.iter().for_each(|(order_id, qty, allocation)| {
                if (qty - allocation <= 0) {
                    order_book.remove_order(CancelOrder {
                        client_id: 0,
                        action: BUY,
                        id: *order_id,
                    });
                } else {
                    order_book.modify_order(BUY, *order_id, qty - allocation)
                }
            });

            ask_allocs.iter().for_each(|(order_id, qty, allocation)| {
                if (qty - allocation <= 0) {
                    order_book.remove_order(CancelOrder {
                        client_id: 0,
                        action: SELL,
                        id: *order_id,
                    });
                } else {
                    order_book.modify_order(SELL, *order_id, qty - allocation)
                }
            });
        }

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
