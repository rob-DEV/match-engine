use std::{collections::HashMap, u128};

use super::util::{self, current_epoch_nano_time};

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct Instrument {
    base: String,
    quote: String,
}

#[derive(Debug)]
pub enum BuyOrSell {
    BUY,
    SELL,
}

#[derive(Debug)]
pub struct Limit {
    price: u32,
    quantity: u32,
    orders: Vec<Order>,
}

impl Limit {
    pub fn new(price: u32) -> Limit {
        Limit {
            price,
            quantity: 0,
            orders: Vec::new(),
        }
    }

    pub fn insert_order(&mut self, order: Order) {
        self.quantity += order.quantity;
        self.orders.push(order);
    }
}

#[derive(Debug)]
pub enum OrderMatchType {
    AON,
    FOK,
}


#[derive(Debug)]
pub struct Order {
    user_id: u64,
    quantity: u32,
    price: u32, // int pricing
    bid_or_offer: BuyOrSell,
    order_match_type: OrderMatchType,
    place_time: u128,
}

impl Order {
    pub fn new(user_id: u64, quantity: u32, price: u32, bid_or_offer: BuyOrSell, order_match_type: OrderMatchType) -> Order {
        Order {
            user_id,
            quantity,
            price,
            bid_or_offer,
            order_match_type,
            place_time: current_epoch_nano_time(),
        }
    }
}

#[derive(Debug)]
pub struct ExecutedOrder {
    buy_user_id: u64,
    sell_user_id: u64,
    order_match_type: OrderMatchType,
    quantity: u32,
    price: u32, // int pricing
    execution_time: u128,
}

impl ExecutedOrder {
    pub fn new(
        buy_user_id: u64,
        sell_user_id: u64,
        order_match_type: OrderMatchType,
        quantity: u32,
        price: u32,
        execution_time: u128,
    ) -> ExecutedOrder {
        ExecutedOrder {
            buy_user_id,
            sell_user_id,
            order_match_type,
            quantity,
            price,
            execution_time,
        }
    }
}

#[derive(Debug)]
pub struct Orderbook {
    bids: HashMap<u32, Limit>,
    offers: HashMap<u32, Limit>,
}

impl Orderbook {
    pub fn new() -> Orderbook {
        Orderbook {
            bids: HashMap::new(),
            offers: HashMap::new(),
        }
    }

    pub fn place_order(&mut self, order: Order) {
        let price_as_map_key = order.price;

        match order.bid_or_offer {
            BuyOrSell::BUY => self
                .bids
                .entry(price_as_map_key)
                .or_insert_with(|| Limit::new(price_as_map_key))
                .insert_order(order),
            BuyOrSell::SELL => self
                .offers
                .entry(price_as_map_key)
                .or_insert_with(|| Limit::new(price_as_map_key))
                .insert_order(order),
        };
    }
}

#[derive(Debug)]
pub struct MatchEngine {
    orderbook: Orderbook,
}

impl MatchEngine {
    pub fn new(orderbook: Orderbook) -> MatchEngine {
        MatchEngine { orderbook }
    }

    pub fn cycle(&mut self) -> Vec<ExecutedOrder> {
        let mut executed_orders: Vec<ExecutedOrder> = Vec::new();
       
        // AON MATCHING - No partial fill and cancelled if no match
        for (_, (b_price, b_limit)) in self.orderbook.bids.iter().enumerate() {
            // Match orders in limit
            let b_limit = b_limit;
            let s_limit = self.orderbook.offers.get(b_price).unwrap();

            for (b_order_index, b_order) in b_limit.orders.iter().enumerate() {
                match b_order.order_match_type {
                    OrderMatchType::AON => aon_match(b_order_index, s_limit, b_order, &mut executed_orders),
                    OrderMatchType::FOK => panic!()
                }
            }
        }

        return executed_orders;
    }
}

fn aon_match(b_order_index: usize, s_limit: &Limit, b_order: &Order, executed_orders: &mut Vec<ExecutedOrder>) {
    let mut b_matched_indexes: Vec<usize> = Vec::new();
    let mut s_matched_indexes: Vec<usize> = Vec::new();

    for (s_order_index, s_order) in s_limit.orders.iter().enumerate() {
        if b_order.quantity == s_order.quantity {
            // Flush from book and create execution records
            // println!(
                // "AON match between {} and {} for {} units at {}",
                // b_order.user_id, s_order.user_id, b_order.quantity, b_order.price
            // );
            b_matched_indexes.push(b_order_index);
            s_matched_indexes.push(s_order_index);
        
            executed_orders.push(ExecutedOrder::new(
                b_order.user_id,
                s_order.user_id,
                OrderMatchType::AON,
                b_order.quantity,
                b_order.price,
                util::current_epoch_nano_time(),
            ))
        }
    }

}
