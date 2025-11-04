use crate::message::GatewayMessage;
use common::domain::domain::{CancelOrder, NewOrder, Side};
use common::domain::messaging::EngineMessage;
use common::domain::order::TimeInForce;
use common::util::time::system_nanos;
use fefix::definitions::fix42::MsgType;
use fefix::prelude::*;
use fefix::tagvalue::{Config, DecodeError, Decoder, Encoder};

pub struct MessageConverter {
    fix_decoder: Decoder,
    fix_encoder: Encoder,
}

impl MessageConverter {
    pub fn new() -> MessageConverter {
        let mut fix_decoder = Decoder::<Config>::new(Dictionary::fix42());
        fix_decoder.config_mut().set_separator(b'|');

        let mut fix_encoder = Encoder::<Config>::default();
        fix_encoder.config_mut().set_separator(b'|');

        MessageConverter {
            fix_decoder,
            fix_encoder,
        }
    }
    pub fn fix_to_in_msg(
        &mut self,
        client_id: u32,
        fix_message_buffer: &[u8],
    ) -> Result<GatewayMessage, DecodeError> {
        let fix_msg = self.fix_decoder.decode(fix_message_buffer)?;

        let fix_msg_type = MsgType::deserialize(fix_msg.fv(fix42::MSG_TYPE).unwrap()).unwrap();

        let msg = match fix_msg_type {
            MsgType::OrderSingle => {
                let fix_msg_px = fix_msg.fv::<u32>(fix44::PRICE).unwrap();
                let fix_msg_qty = fix_msg.fv::<u32>(fix44::ORDER_QTY).unwrap();
                let fix_msg_side = fix_msg.fv::<&str>(fix44::SIDE).unwrap();

                let mut order_side = Side::BUY;
                if fix_msg_side == "2" {
                    order_side = Side::SELL;
                }

                GatewayMessage::LimitOrder(NewOrder {
                    client_id,
                    order_side,
                    px: fix_msg_px,
                    qty: fix_msg_qty,
                    time_in_force: TimeInForce::GTC,
                    timestamp: system_nanos(),
                })
            }
            MsgType::OrderCancelRequest => {
                let fix_msg_side = fix_msg.fv::<&str>(fix44::SIDE).unwrap();
                let fix_msg_order_id = fix_msg.fv::<u32>(fix44::ORDER_ID).unwrap();

                let mut order_side = Side::BUY;
                if fix_msg_side == "2" {
                    order_side = Side::SELL;
                }

                GatewayMessage::CancelOrder(CancelOrder {
                    client_id,
                    order_side,
                    order_id: fix_msg_order_id,
                })
            }

            _ => {
                unimplemented!();
            }
        };

        Ok(msg)
    }

    pub fn engine_msg_out_to_fix(&mut self, engine_msg_out: EngineMessage) -> Vec<u8> {
        let mut out_buffer = vec![0; 128];

        match engine_msg_out {
            EngineMessage::NewOrderAck(_) => {
                let mut out_fix =
                    self.fix_encoder
                        .start_message(b"FIX.4.4", &mut out_buffer, b"ExecutionReport");
            }
            EngineMessage::RejectionMessage(_) => {
                let mut out_fix =
                    self.fix_encoder
                        .start_message(b"FIX.4.4", &mut out_buffer, b"Reject");
                out_fix.set(fix44::SESSION_REJECT_REASON, 7)
            }
            EngineMessage::EngineError(_) => {
                let mut out_fix =
                    self.fix_encoder
                        .start_message(b"FIX.4.4", &mut out_buffer, b"Reject");
                out_fix.set(fix44::SESSION_REJECT_REASON, 99)
            }
            _ => {
                unimplemented!()
            }
        };

        out_buffer
    }
}
