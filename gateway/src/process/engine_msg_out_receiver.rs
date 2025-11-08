use common::message::execution_report::{ExecType, ExecutionReport};
use common::network::mutlicast::multicast_receiver;
use common::transport::sequenced_message::EngineMessage;
use common::transport::sequenced_multicast_receiver::SequencedMulticastReceiver;
use common::transport::transport_constants::MSG_OUT_CHANNEL;
use common::util::time::system_nanos;
use dashmap::DashMap;
use std::error::Error;
use std::sync::mpsc::Sender;
use std::sync::Arc;

pub fn initialize_engine_msg_out_receiver(
    engine_msg_out_port: u16,
    session_data_tx: Arc<DashMap<u32, Sender<EngineMessage>>>,
) -> Result<(), Box<dyn Error>> {
    let udp_socket = multicast_receiver(engine_msg_out_port);

    let mut multicast_receiver =
        SequencedMulticastReceiver::new(Box::from(udp_socket), MSG_OUT_CHANNEL);

    println!(
        "Initialized MSG_OUT -> Gateway multicast on port {}",
        engine_msg_out_port
    );

    loop {
        let session_data = session_data_tx.clone();

        if let Some(outbound_engine_message) = multicast_receiver.try_recv() {
            let outbound_message_type = &outbound_engine_message.message;

            match outbound_message_type {
                EngineMessage::NewOrderAck(new_ack) => {
                    let client_id = new_ack.client_id;

                    let session_state = session_data.get_mut(&client_id).unwrap();
                    session_state.send(outbound_engine_message.message).unwrap()
                }
                EngineMessage::CancelOrderAck(cancel_ack) => {
                    let client_id = cancel_ack.client_id;
                    let session_state = session_data.get_mut(&client_id).unwrap();
                    session_state.send(outbound_engine_message.message).unwrap()
                }
                EngineMessage::TradeExecution(execution) => {
                    let bid_client_id = execution.bid_client_id;
                    let ask_client_id = execution.ask_client_id;

                    let ask_exec = EngineMessage::TradeExecution(ExecutionReport {
                        trade_id: execution.trade_id,
                        trade_seq: execution.trade_seq,
                        bid_client_id: execution.bid_client_id,
                        bid_order_id: execution.bid_order_id,
                        bid_order_px: execution.bid_order_px,
                        bid_fill_type: execution.bid_fill_type,
                        ask_client_id: execution.ask_client_id,
                        ask_order_id: execution.ask_order_id,
                        ask_order_px: execution.ask_order_px,
                        ask_fill_type: execution.ask_fill_type,
                        exec_qty: execution.exec_qty,
                        exec_px: execution.exec_px,
                        exec_type: ExecType::MatchEvent,
                        execution_time: system_nanos(),
                    });

                    let bid_tx = session_data.get(&bid_client_id).unwrap();
                    bid_tx.send(outbound_engine_message.message).unwrap();

                    let ask_tx = session_data.get(&ask_client_id).unwrap();
                    ask_tx.send(ask_exec).unwrap();
                }
                _ => {
                    unimplemented!()
                }
            }
        }
    }

    Ok(())
}
