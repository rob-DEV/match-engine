use common::engine::{InboundEngineMessage, InboundMessage, Logon, NewOrder, OrderAction, OutboundEngineMessage, OutboundMessage};
use fefix::definitions::fix42::MsgType;
use fefix::prelude::*;
use fefix::tagvalue::{Config, DecodeError, Decoder, Encoder};
use rand::random;

pub struct MessageConverter {
    fix_decoder: Decoder<>,
    fix_encoder: Encoder<>,
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
    pub fn fix_to_in_msg(&mut self, fix_message_buffer: &[u8]) -> Result<InboundEngineMessage, DecodeError> {
        let fix_msg = self.fix_decoder.decode(fix_message_buffer)?;

        let fix_msg_type = MsgType::deserialize(fix_msg.fv(fix42::MSG_TYPE).unwrap()).unwrap();

        let msg = match fix_msg_type {
            MsgType::Logon => {
                println!("Logon");
                let heartbeat_int: u32 = fix_msg.fv(fix42::HEART_BT_INT).unwrap();

                InboundEngineMessage {
                    seq_num: 0,
                    inbound_message: InboundMessage::Logon(Logon {
                        heartbeat_sec: heartbeat_int
                    }),
                }
            }
            MsgType::Logout => {
                println!("Logout");
                InboundEngineMessage {
                    seq_num: 0,
                    inbound_message: InboundMessage::Logon(Logon {
                        heartbeat_sec: 0
                    }),
                }
            }
            MsgType::OrderSingle => {
                println!("NewOrderSingle");

                let fix_msg_px = fix_msg.fv::<u32>(fix44::PRICE).unwrap();
                let fix_msg_qty = fix_msg.fv::<u32>(fix44::ORDER_QTY).unwrap();
                let fix_msg_side = fix_msg.fv::<&str>(fix44::SIDE).unwrap();

                let mut order_action = OrderAction::BUY;
                if fix_msg_side == "2" { order_action = OrderAction::SELL; }

                InboundEngineMessage {
                    seq_num: 0,
                    inbound_message: InboundMessage::NewOrder(NewOrder {
                        order_action,
                        px: fix_msg_px,
                        qty: fix_msg_qty,
                    }),
                }
            }
            MsgType::OrderCancelRequest => {
                println!("OrderCancelRequest");
                InboundEngineMessage {
                    seq_num: 0,
                    inbound_message: InboundMessage::Logon(Logon {
                        heartbeat_sec: 0
                    }),
                }
            }

            _ => {
                let rand = random::<u32>() % 2 == 0;

                let order_action;
                if rand {
                    order_action = OrderAction::BUY
                } else {
                    order_action = OrderAction::SELL
                }

                InboundEngineMessage {
                    seq_num: 0,
                    inbound_message: InboundMessage::NewOrder(NewOrder {
                        order_action,
                        px: 30,
                        qty: 30,
                    }),
                }
            }
        };

        //
        Ok(msg)
    }

    pub fn engine_msg_out_to_fix(&mut self, engine_msg_out: OutboundEngineMessage) -> Vec<u8> {
        let mut out_buffer = vec![0; 2048];

        match engine_msg_out.outbound_message {
            OutboundMessage::NewOrderAck(_) => {
                let mut out_fix = self.fix_encoder.start_message(b"FIX.4.4", &mut out_buffer, b"ExecutionReport");
            }
            OutboundMessage::RejectionMessage(_) => {
                let mut out_fix = self.fix_encoder.start_message(b"FIX.4.4", &mut out_buffer, b"Reject");
                out_fix.set(fix44::SESSION_REJECT_REASON, 7)
            }
            OutboundMessage::EngineError(_) => {
                let mut out_fix = self.fix_encoder.start_message(b"FIX.4.4", &mut out_buffer, b"Reject");
                out_fix.set(fix44::SESSION_REJECT_REASON, 99)
            }
            _ => {
                unimplemented!()
            }
        };

        out_buffer
    }
}