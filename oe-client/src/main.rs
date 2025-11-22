use common::transport::sequenced_message::EngineMessage;
use common::types::cancel_order::CancelOrderRequest;
use common::types::instrument::Instrument;
use common::types::order::{OrderRequest, TimeInForce};
use common::types::side::Side;
use common::util::time::system_nanos;
use rand::random;
use std::error::Error;
use std::io::{BufRead, Read, Write};
use std::net::TcpStream;
use std::process::exit;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::atomic::{AtomicBool, AtomicU32};
use std::sync::mpsc::Receiver;
use std::thread;

fn writer(mut write_stream: TcpStream, sequenced_message_store: Receiver<EngineMessage>) {
    let mut count = 0;
    while let Ok(message) = sequenced_message_store.recv() {
        let serialized = common::serialize::serialize::as_bytes(&message);

        let packet_len = serialized.len();
        if let Err(err) = write_stream.write_all(&packet_len.to_be_bytes()) {
            println!("Gateway write error: {:?}", err);
        }

        if let Err(err) = write_stream.write_all(&serialized) {
            println!("Gateway write error: {:?}", err);
            break;
        }
        count += 1;
    }
}
static SHOULD_LOG: AtomicBool = AtomicBool::new(true);
static CLIENT_ID: AtomicU32 = AtomicU32::new(0);

fn reader(mut read_stream: TcpStream) {
    let mut buffer: [u8; 4096] = [0; 4096];

    loop {
        let mut len_buf = [0u8; 4];
        if read_stream.read_exact(&mut len_buf).is_err() {
            return; // disconnected
        }

        let frame_len = u32::from_be_bytes(len_buf) as usize;

        if frame_len > 0 {
            if frame_len > buffer.len() {
                println!("Frame too large");
                return;
            }

            read_stream.read_exact(&mut buffer[..frame_len]).unwrap();

            // if SHOULD_LOG.load(Ordering::Relaxed) {
            // println!("{:?}", buffer);
            // }
        }
    }
}

fn client_connection(sequenced_message_store: Receiver<EngineMessage>) {
    let mut tcp_stream = TcpStream::connect("127.0.0.1:3001")
        .map_err(|e| "Failed to connect to the market-gateway server")
        .unwrap();

    let read_stream = tcp_stream.try_clone().unwrap();
    let write_stream = tcp_stream.try_clone().unwrap();

    let writer_thread = thread::spawn(move || {
        writer(write_stream, sequenced_message_store);
    });

    let reader_thread = thread::spawn(move || {
        reader(read_stream);
    });

    writer_thread.join().unwrap();
    reader_thread.join().unwrap();
}

#[derive(Debug)]
enum Command {
    Buy(u32, u32),
    Sell(u32, u32),
    Cancel(bool, u32),
    Perf(bool, u32),
    Quit,
}

fn parse(input: String) -> Result<Command, ()> {
    let lower = input.to_lowercase();
    let tokens = lower.split(" ").collect::<Vec<&str>>();

    match tokens[0] {
        "buy" | "b" => {
            let px = tokens[1].parse::<u32>().unwrap();
            let qty = tokens[2].parse::<u32>().unwrap();
            Ok(Command::Buy(px, qty))
        }
        "sell" | "s" => {
            let px = tokens[1].parse::<u32>().unwrap();
            let qty = tokens[2].parse::<u32>().unwrap();
            Ok(Command::Sell(px, qty))
        }
        "cancel" | "c" => {
            let side = tokens[1];
            let order_id = tokens[2].parse::<u32>().unwrap();
            if side == "b" {
                Ok(Command::Cancel(true, order_id))
            } else {
                Ok(Command::Cancel(false, order_id))
            }
        }
        "perf" | "p" => {
            let side = tokens[1];
            let batch_size = tokens[2].parse::<u32>().unwrap();
            if side == "b" {
                Ok(Command::Perf(true, batch_size))
            } else {
                Ok(Command::Perf(false, batch_size))
            }
        }
        "quit" | "q" => Ok(Command::Quit),
        _ => Err(()),
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    CLIENT_ID.store(random::<u32>(), Relaxed);
    let (sender, receiver) = std::sync::mpsc::channel::<EngineMessage>();
    let oe_client_thread = thread::spawn(move || client_connection(receiver));

    println!("-----------------");
    println!("OE CLIENT\nBUY px qty\nSELL px qty\nCANCEL side order_id\nPERF n_orders\nQUIT");
    println!("-----------------");

    let quit = false;
    while !quit {
        println!("Enter input:");

        let mut line = String::new();
        std::io::stdin().read_line(&mut line).unwrap();

        if let Ok(command) = parse(line.trim().to_string()) {
            let order;
            match command {
                Command::Buy(px, qty) => {
                    order = build_nos(true, px, qty);
                    sender.clone().send(order).expect("TODO: panic types");
                }
                Command::Sell(px, qty) => {
                    order = build_nos(false, px, qty);
                    sender.clone().send(order).expect("TODO: panic types");
                }
                Command::Cancel(is_buy, order_id) => {
                    order = build_cancel(is_buy, order_id);
                    sender.clone().send(order).expect("TODO: panic types");
                }
                Command::Perf(is_buy, batch_size) => {
                    SHOULD_LOG.store(false, std::sync::atomic::Ordering::Release);
                    for _ in 0..batch_size {
                        let order;
                        let px = (random::<u32>() % 100) + 1;
                        let qty = (random::<u32>() % 100) + 1;

                        order = build_nos(is_buy, px, qty);
                        sender.send(order).expect("TODO: panic types");
                    }

                    println!("Perf done!");
                    SHOULD_LOG.store(true, std::sync::atomic::Ordering::Release);
                }
                Command::Quit => {
                    exit(0);
                }
            }
        } else {
            println!("Not a known command!");
        }
    }

    oe_client_thread.join().unwrap();
    Ok(())
}

fn build_nos(is_buy: bool, px: u32, qty: u32) -> EngineMessage {
    let side = match is_buy {
        true => Side::Buy,
        false => Side::Sell,
    };

    EngineMessage::NewOrder(OrderRequest {
        client_id: CLIENT_ID.load(Relaxed),
        instrument: Instrument::str_to_fixed_char_buffer("BTC-USD"),
        order_side: side,
        px,
        qty,
        time_in_force: TimeInForce::GTC,
        timestamp: system_nanos(),
    })
}

fn build_cancel(is_buy: bool, order_id: u32) -> EngineMessage {
    let side = match is_buy {
        true => Side::Buy,
        false => Side::Sell,
    };

    EngineMessage::CancelOrder(CancelOrderRequest {
        client_id: CLIENT_ID.load(Relaxed),
        order_side: side,
        order_id,
        instrument: Instrument::str_to_fixed_char_buffer("BTC-USD"),
    })
}
