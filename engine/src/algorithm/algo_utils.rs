use crate::book::order_book::Price;
use crate::domain::execution::Execution;
use crate::domain::order::LimitOrder;
use common::message::execution::ExecType;
use common::message::side::Side;
use common::util::time::system_nanos;
use rand::random;

pub fn best_prices_cross(order: &mut LimitOrder, best_px: Price) -> bool {
    let px_cross = match order.side {
        Side::BUY => order.px >= best_px,
        Side::SELL => order.px <= best_px,
    };
    px_cross
}

pub fn build_fill_execution(
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

    let exec_type = if order.qty == fill_qty {
        ExecType::Fill
    } else {
        ExecType::PartialFill
    };

    Execution {
        id: random::<u32>(),
        bid,
        ask,
        exec_qty: fill_qty,
        exec_type,
        execution_time: system_nanos(),
    }
}
