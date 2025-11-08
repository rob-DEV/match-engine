// use crate::algorithm::algo_utils::{best_prices_cross, build_fill_execution, build_self_match_prevention_execution};
// use crate::algorithm::match_strategy::MatchStrategy;
// use crate::book::book::Book;
// use crate::book::order_book::LimitOrderBook;
// use crate::book::price_level::PriceLevel;
// use crate::domain::execution::Execution;
// use crate::domain::order::LimitOrder;
// use common::message::side::Side;
// use std::collections::HashMap;
//
// pub struct ProRataMatchStrategy;
//
// impl MatchStrategy for ProRataMatchStrategy {
//     fn new() -> Self {
//         ProRataMatchStrategy {}
//     }
//
//     fn match_orders(
//         &mut self,
//         order_book: &mut LimitOrderBook,
//         order: &mut LimitOrder,
//         executions_buffer: &mut Vec<Execution>,
//     ) -> usize {
//         let mut num_executions: usize = 0;
//
//         let (book_side, opposite_book_side) = match order.side {
//             Side::BUY => (&mut order_book.bids, &mut order_book.asks),
//             Side::SELL => (&mut order_book.asks, &mut order_book.bids),
//         };
//
//         loop {
//             let Some(best_px) = opposite_book_side.best_price() else {
//                 break; // No liquidity - skip match, add to orderbook
//             };
//
//             if best_prices_cross(order, best_px) {
//                 let opposite_price_level =
//                     match opposite_book_side.price_level_map.get_mut(&best_px) {
//                         Some(l) => l,
//                         None => break,
//                     };
//
//                 let price_level_total_qty = opposite_price_level.total_qty;
//                 let matched_qty = order.qty.min(price_level_total_qty);
//                 let pro_rata_allocations = Self::pro_rata_allocate_fills(
//                     opposite_price_level,
//                     &opposite_book_side.order_map,
//                     matched_qty,
//                 );
//
//                 let mut remaining_qty = matched_qty;
//                 let mut pro_rata_allocation_idx = 0;
//                 // Match loop
//                 while remaining_qty > 0 && !opposite_price_level.is_empty() {
//                     let resting_id = opposite_price_level.front().copied().unwrap();
//
//                     let resting_order = match opposite_book_side.order_map.get_mut(&resting_id) {
//                         Some(o) => o,
//                         None => {
//                             opposite_price_level.pop_front(); // cleanup orphan id
//                             continue;
//                         }
//                     };
//
//                     let fill_qty = pro_rata_allocations[pro_rata_allocation_idx];
//                     if fill_qty == 0 {
//                         opposite_price_level.pop_front();
//                         opposite_book_side.order_map.remove(&resting_id);
//                         continue;
//                     }
//
//                     // Adjust quantities
//                     // This needs abstracted
//                     order.qty -= fill_qty;
//                     resting_order.qty -= fill_qty;
//                     opposite_price_level.total_qty -= fill_qty;
//                     opposite_book_side.total_qty -= fill_qty;
//
//                     // Record execution
//                     executions_buffer.push(build_fill_execution(order, resting_order, fill_qty));
//
//                     remaining_qty -= fill_qty;
//                     pro_rata_allocation_idx += 1;
//                     num_executions += 1;
//
//                     // Remove fully filled book order
//                     if resting_order.qty == 0 {
//                         opposite_book_side.order_map.remove(&resting_id);
//                         opposite_price_level.pop_front();
//                         opposite_book_side.num_orders -= 1;
//                     }
//                 }
//
//                 // Remove empty price level
//                 if opposite_price_level.is_empty() {
//                     opposite_book_side.price_level_map.remove(&best_px);
//                 }
//
//                 if order.qty == 0 {
//                     break;
//                 }
//             } else {
//                 break;
//             }
//         }
//
//         executions_buffer.len()
//     }
// }
//
// impl ProRataMatchStrategy {
//     fn pro_rata_allocate_fills(
//         price_level: &PriceLevel,
//         order_map: &HashMap<u32, LimitOrder>,
//         matched_qty: u32,
//     ) -> Vec<u32> {
//         let mut allocations: Vec<u32> = price_level
//             .order_ids()
//             .map(|order_id| {
//                 (order_map.get(order_id).unwrap().qty as f64 * matched_qty as f64
//                     / price_level.total_qty as f64)
//                     .floor() as u32
//             })
//             .collect();
//
//         let allocated_qty = allocations.iter().sum::<u32>();
//
//         if allocated_qty != matched_qty {
//             let adjustment = matched_qty - allocated_qty;
//             // println!(
//             //     "Allocations require adjustment: expected: {}, actual: {} = adjustment: {}",
//             //     matched_qty, allocated_qty, adjustment
//             // );
//
//             let max_allocation_idx = allocations
//                 .iter()
//                 .enumerate()
//                 .max_by_key(|(_i, &val)| val)
//                 .map(|(i, _val)| i)
//                 .unwrap();
//
//             allocations[max_allocation_idx] += adjustment;
//         }
//
//         assert_eq!(allocations.iter().sum::<u32>(), matched_qty);
//
//         allocations
//     }
// }
