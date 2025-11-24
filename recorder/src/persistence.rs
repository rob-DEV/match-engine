use crate::persistable_entities::{OrderRecord, TradeRecord};
use common::transport::sequenced_message::{EngineMessage, SequencedEngineMessage};
use sqlx::{Pool, Postgres};

pub struct Persistence {
    db: Pool<Postgres>,
}

impl Persistence {
    pub fn new(db: Pool<Postgres>) -> Self {
        Persistence { db }
    }

    pub async fn persist_event(
        &self,
        engine_message: SequencedEngineMessage,
    ) -> anyhow::Result<()> {
        println!("Got outbound engine message");

        match engine_message.message {
            EngineMessage::NewOrderAck(new_order_ack) => {
                let order_record = OrderRecord {
                    order_id: new_order_ack.order_id as i32,
                    client_id: new_order_ack.client_id as i32,
                    instrument: new_order_ack.instrument,
                    side: new_order_ack.side as i16,
                    px: new_order_ack.px as i32,
                    qty: new_order_ack.qty as i32,
                    qty_rem: new_order_ack.qty_rem as i32,
                    time_in_force: new_order_ack.time_in_force as i16,
                    ack_time: new_order_ack.ack_time as i64,
                };

                OrderRecord::insert(&self.db, &order_record).await?;
            }
            EngineMessage::CancelOrderAck(cancel_order_ack) => {}
            EngineMessage::TradeExecution(trade_execution) => {
                let trade_record = TradeRecord {
                    trade_id: trade_execution.trade_id as i32,
                    bid_client_id: trade_execution.bid_client_id as i32,
                    bid_order_id: trade_execution.bid_order_id as i32,
                    bid_order_px: trade_execution.bid_order_px as i32,
                    bid_fill_type: trade_execution.bid_fill_type as i16,
                    ask_client_id: trade_execution.ask_client_id as i32,
                    ask_order_id: trade_execution.ask_order_id as i32,
                    ask_order_px: trade_execution.ask_order_px as i32,
                    ask_fill_type: trade_execution.ask_fill_type as i16,
                    instrument: trade_execution.instrument,
                    exec_px: trade_execution.exec_px as i32,
                    exec_qty: trade_execution.exec_qty as i32,
                    exec_type: trade_execution.exec_type as i16,
                    exec_ns: trade_execution.exec_ns as i64,
                };

                TradeRecord::insert(&self.db, &trade_record).await?;
            }

            _ => {}
        }

        Ok(())
    }
}
