use common::domain::domain::TradeExecution;
use common::domain::messaging::{EngineMessage, SequencedEngineMessage};
use common::network::network_constants::MAX_UDP_PACKET_SIZE;
use common::network::udp_socket::multicast_udp_socket;
use std::collections::HashMap;
use std::error::Error;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

pub fn initialize_engine_msg_out_receiver(
    engine_msg_out_port: u16,
    session_data_tx: Arc<Mutex<HashMap<u32, Sender<EngineMessage>>>>,
) -> Result<(), Box<dyn Error>> {
    let udp_socket = multicast_udp_socket(engine_msg_out_port, true);
    let mut buffer = [0; MAX_UDP_PACKET_SIZE];

    println!(
        "Initialized MSG_OUT -> Gateway multicast on port {}",
        engine_msg_out_port
    );

    loop {
        match udp_socket.recv_from(&mut buffer) {
            Ok((size, _)) => {
                let outbound_engine_message: SequencedEngineMessage =
                    bitcode::decode(&buffer[..size]).unwrap();

                let outbound_message_type = &outbound_engine_message.message;

                let mut session_data = session_data_tx.lock().unwrap();
                match outbound_message_type {
                    EngineMessage::NewOrderAck(new_ack) => {
                        // println!("New order ack: {:?}", new_ack);
                        let client_id = new_ack.client_id;

                        let session_state = session_data.get_mut(&client_id).unwrap();
                        session_state.send(outbound_engine_message.message).unwrap()
                    }
                    EngineMessage::CancelOrderAck(cancel_ack) => {
                        // println!("Cancel order ack: {:?}", cancel_ack);
                        let client_id = cancel_ack.client_id;
                        let session_state = session_data.get_mut(&client_id).unwrap();
                        session_state.send(outbound_engine_message.message).unwrap()
                    }
                    EngineMessage::TradeExecution(execution) => {
                        let bid_client_id = execution.bid_client_id;
                        let ask_client_id = execution.ask_client_id;

                        let ask_exec = EngineMessage::TradeExecution(TradeExecution {
                            trade_id: execution.trade_id,
                            trade_seq: execution.trade_seq,
                            bid_client_id: execution.bid_client_id,
                            ask_client_id: execution.ask_client_id,
                            bid_order_id: execution.bid_order_id,
                            ask_order_id: execution.ask_order_id,
                            fill_qty: execution.fill_qty,
                            px: execution.px,
                            execution_time: execution.execution_time,
                        });

                        // println!("Trade execution: {:?}", execution);
                        let bid_tx = session_data.get(&bid_client_id).unwrap();
                        bid_tx.send(outbound_engine_message.message).unwrap();

                        // println!("Trade execution: {:?}", ask_exec);
                        let ask_tx = session_data.get(&ask_client_id).unwrap();
                        ask_tx.send(ask_exec).unwrap();
                    }
                    _ => {
                        unimplemented!()
                    }
                }
            }
            Err(_) => {}
        }
    }

    Ok(())
}
