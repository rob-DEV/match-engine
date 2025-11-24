use common::types::execution_report::{ExecType, FillType};
use common::types::instrument::Instrument;
use common::types::order::TimeInForce;
use common::types::side::Side;
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct OrderRecord {
    pub order_id: i32,
    pub client_id: i32,
    pub instrument: [u8; 16],
    pub side: i16,
    pub px: i32,
    pub qty: i32,
    pub qty_rem: i32,
    pub time_in_force: i16,
    pub ack_time: i64,
}

impl OrderRecord {
    pub async fn insert(pool: &sqlx::PgPool, o: &OrderRecord) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
        INSERT INTO orders (
            order_id, client_id, instrument, side,
            px, qty, qty_rem, time_in_force, ack_time
        )
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)
        "#,
            o.order_id,
            o.client_id,
            bytes_to_str(&o.instrument),
            o.side,
            o.px,
            o.qty,
            o.qty_rem,
            o.time_in_force,
            o.ack_time
        )
        .execute(pool)
        .await?;
        Ok(())
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct TradeRecord {
    pub trade_id: i32,
    pub bid_client_id: i32,
    pub bid_order_id: i32,
    pub bid_order_px: i32,
    pub bid_fill_type: i16,
    pub ask_client_id: i32,
    pub ask_order_id: i32,
    pub ask_order_px: i32,
    pub ask_fill_type: i16,
    pub instrument: [u8; 16],
    pub exec_px: i32,
    pub exec_qty: i32,
    pub exec_type: i16,
    pub exec_ns: i64,
}

impl TradeRecord {
    pub async fn insert(pool: &sqlx::PgPool, t: &TradeRecord) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
        INSERT INTO trades (
            trade_id,
            bid_client_id, bid_order_id, bid_order_px, bid_fill_type,
            ask_client_id, ask_order_id, ask_order_px, ask_fill_type,
            instrument,
            exec_px, exec_qty, exec_type, exec_ns
        ) VALUES (
            $1,$2,$3,$4,$5,
            $6,$7,$8,$9,
            $10,
            $11,$12,$13,$14
        )
        "#,
            t.trade_id,
            t.bid_client_id,
            t.bid_order_id,
            t.bid_order_px,
            t.bid_fill_type as i16,
            t.ask_client_id,
            t.ask_order_id,
            t.ask_order_px,
            t.ask_fill_type as i16,
            bytes_to_str(&t.instrument),
            t.exec_px,
            t.exec_qty,
            t.exec_type as i16,
            t.exec_ns
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}

fn bytes_to_str(bytes: &[u8]) -> &str {
    let len = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    std::str::from_utf8(&bytes[..len]).unwrap()
}
