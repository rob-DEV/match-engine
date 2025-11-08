use crate::book::order_book::Price;
use crate::domain::order::LimitOrder;
use common::message::execution_report::{ExecType, ExecutionReport, FillType};
use common::message::side::Side;
use common::util::time::system_nanos;

pub fn best_prices_cross(order: &LimitOrder, best_px: Price) -> bool {
    let px_cross = match order.side {
        Side::BUY => order.px >= best_px,
        Side::SELL => order.px <= best_px,
    };
    px_cross
}

pub fn traders_will_self_match(order: &LimitOrder, resting_order: &LimitOrder) -> bool {
    order.client_id == resting_order.client_id
}

pub fn build_fill_execution(
    order: &mut LimitOrder,
    resting_order: &mut LimitOrder,
    exec_px: u32,
    exec_qty: u32,
) -> ExecutionReport {
    let bid;
    let ask;

    match order.side {
        Side::BUY => {
            bid = order;
            ask = resting_order;
        }
        Side::SELL => {
            bid = resting_order;
            ask = order;
        }
    }

    let bid_fill_type = if bid.qty == exec_qty {
        FillType::FullFill
    } else {
        FillType::PartialFill
    };

    let ask_fill_type = if ask.qty == exec_qty {
        FillType::FullFill
    } else {
        FillType::PartialFill
    };

    ExecutionReport {
        trade_id: 0,
        trade_seq: 0,
        bid_client_id: bid.client_id,
        bid_order_id: bid.id,
        bid_order_px: bid.px,
        bid_fill_type,
        ask_client_id: ask.client_id,
        ask_order_id: ask.id,
        ask_order_px: ask.px,
        ask_fill_type,
        exec_qty,
        exec_px,
        exec_type: ExecType::MatchEvent,
        execution_time: system_nanos(),
    }
}

pub fn build_self_match_prevention_execution(resting_order: &mut LimitOrder) -> ExecutionReport {
    match resting_order.side {
        Side::BUY => ExecutionReport {
            trade_id: 0,
            trade_seq: 0,
            bid_client_id: resting_order.client_id,
            bid_order_id: resting_order.id,
            bid_order_px: resting_order.px,
            bid_fill_type: FillType::NoFill,
            ask_client_id: 0,
            ask_order_id: 0,
            ask_order_px: 0,
            ask_fill_type: FillType::NoFill,
            exec_px: resting_order.px,
            exec_qty: resting_order.qty,
            exec_type: ExecType::SelfMatchPrevented,
            execution_time: system_nanos(),
        },
        Side::SELL => ExecutionReport {
            trade_id: 0,
            trade_seq: 0,
            bid_client_id: 0,
            bid_order_id: 0,
            bid_order_px: 0,
            bid_fill_type: FillType::NoFill,
            ask_client_id: resting_order.client_id,
            ask_order_id: resting_order.id,
            ask_order_px: resting_order.px,
            ask_fill_type: FillType::NoFill,
            exec_qty: resting_order.qty,
            exec_type: ExecType::SelfMatchPrevented,
            exec_px: resting_order.px,
            execution_time: system_nanos(),
        },
    }
}
