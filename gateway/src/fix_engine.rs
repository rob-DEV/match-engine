use common::engine::{InboundEngineMessage, InboundMessage, NewOrder, OrderAction, OutboundEngineMessage, OutboundMessage};
use fefix::prelude::*;
use fefix::tagvalue::{Config, DecodeError, Decoder, Encoder, Message};

pub struct FixEngine {
    fix_decoder: Decoder<>,
    fix_encoder: Encoder<>
}

impl FixEngine {
    pub fn new() -> FixEngine {
        let mut fix_decoder = Decoder::<Config>::new(Dictionary::fix44());
        fix_decoder.config_mut().set_separator(b'|');

        let mut fix_encoder = Encoder::<Config>::default();
        fix_encoder.config_mut().set_separator(b'|');

        FixEngine {
            fix_decoder,
            fix_encoder
        }
    }
    pub fn fix_to_inbound_engine_message(&mut self, fix_message_buffer: &[u8]) -> Result<InboundMessage, DecodeError> {
        let fix_parse_result = self.fix_decoder.decode(fix_message_buffer);

        match fix_parse_result {
            Ok(msg) => {
                Ok(InboundMessage::NewOrder {
                    0: NewOrder {
                        order_action: OrderAction::BUY,
                        px: 0,
                        qty: 0,
                    },
                })
            },
            Err(err) => Err(err)
        }
    }

    pub fn outbound_engine_message_to_fix(&mut self, outbound_engine_message: OutboundEngineMessage) -> Vec<u8> {
        let mut out_buffer = vec![0; 2048];

        match outbound_engine_message.outbound_message {
            OutboundMessage::NewOrderAck(_) => {
                let mut out_fix = self.fix_encoder.start_message(b"FIX.4.4", &mut out_buffer, b"ExecutionReport");

            }
            OutboundMessage::CancelOrderAck(_) => {
                unimplemented!()
            }
            OutboundMessage::RejectionMessage(_) => {
                let mut out_fix = self.fix_encoder.start_message(b"FIX.4.4", &mut out_buffer, b"Reject");
                out_fix.set(fix44::SESSION_REJECT_REASON, 7)
            }
            OutboundMessage::EngineError(_) => {
                let mut out_fix = self.fix_encoder.start_message(b"FIX.4.4", &mut out_buffer, b"Reject");
                out_fix.set(fix44::SESSION_REJECT_REASON,  99)
            }
        };

        out_buffer
    }
}