use fefix::prelude::*;
use fefix::tagvalue::{Config, Encoder};
use rand::random;
use std::error::Error;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::process::exit;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::Receiver;
use std::thread;

fn writer(mut write_stream: TcpStream, sequenced_message_store: Receiver<String>) {
    let mut count = 0;
    while let Ok(message) = sequenced_message_store.recv() {
        if write_stream.write_all(message.as_bytes()).is_err() {
            break;
        }
        count += 1;
    }
}
static SHOULD_LOG: AtomicBool = AtomicBool::new(true);

fn reader(read_stream: TcpStream) {
    let mut buf_reader = BufReader::new(read_stream);
    let mut line = String::new();
    loop {
        line.clear();
        let bytes_read = buf_reader.read_line(&mut line).unwrap();

        if bytes_read == 0 {
            println!("Client disconnected!");
            exit(0);
        }
    }
}

fn client_connection(sequenced_message_store: Receiver<String>) {
    let mut tcp_stream = TcpStream::connect("127.0.0.1:3001").map_err(|e| { "Failed to connect to the gateway server" }).unwrap();

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
    Perf(u32),
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
            let batch_size = tokens[1].parse::<u32>().unwrap();
            Ok(Command::Perf(batch_size))
        }
        "quit" | "q" => { Ok(Command::Quit) }
        _ => { Err(()) }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let (sender, receiver) = std::sync::mpsc::channel::<String>();
    let oe_client_thread = thread::spawn(move || client_connection(receiver));

    println!("-----------------");
    println!("OE CLIENT\nBUY px qty\nSELL px qty\nPERF n_orders\nQUIT px qty");
    println!("-----------------");

    let quit = false;
    while !quit {
        println!("Enter input:");

        let mut line = String::new();
        std::io::stdin().read_line(&mut line).unwrap();

        if let Ok(command) = parse(line.trim().to_string()) {
            let fix_message;
            match command {
                Command::Buy(px, qty) => {
                    fix_message = build_nos(true, px, qty);
                    sender.clone().send(fix_message.to_string() + "\n").expect("TODO: panic message");
                }
                Command::Sell(px, qty) => {
                    fix_message = build_nos(false, px, qty);
                    sender.clone().send(fix_message.to_string() + "\n").expect("TODO: panic message");
                }
                Command::Cancel(is_buy, order_id) => {
                    fix_message = build_cancel(is_buy, order_id);
                    sender.clone().send(fix_message.to_string() + "\n").expect("TODO: panic message");
                }
                Command::Perf(batch_size) => {
                    SHOULD_LOG.store(false, std::sync::atomic::Ordering::Relaxed);
                    for _ in 0..batch_size {
                        let order_fix;
                        let px = (random::<u32>() % 100) + 1;
                        let qty = (random::<u32>() % 100) + 1;

                        if random::<u32>() % 2 == 0 {
                            order_fix = build_nos(true, px, qty);
                        } else {
                            order_fix = build_nos(false, px, qty);
                        }

                        sender.send(order_fix.to_string() + "\n").expect("TODO: panic message");
                    }

                    println!("Perf done!");
                    SHOULD_LOG.store(true, std::sync::atomic::Ordering::Relaxed);

                }
                Command::Quit => { exit(0); }
            }
        } else {
            println!("Not a known command!");
        }
    }

    oe_client_thread.join().unwrap();
    Ok(())
}

fn build_nos(is_buy: bool, px: u32, qty: u32) -> String {
    let mut encoder = Encoder::<Config>::default();
    encoder.config_mut().set_separator(b'|');
    let mut buffer = Vec::new();
    let mut msg = encoder.start_message(b"FIX.4.2", &mut buffer, b"D");
    msg.set(fix44::MSG_SEQ_NUM, 215);
    msg.set(fix44::SENDER_COMP_ID, "CLIENT12");
    msg.set(fix44::TARGET_COMP_ID, "B");
    msg.set(fix44::ACCOUNT, "TestClient");
    msg.set(fix44::CL_ORD_ID, "13346");
    msg.set(fix44::ORD_TYPE, fix44::OrdType::Limit);
    msg.set(fix44::PRICE, px);
    msg.set(fix44::ORDER_QTY, qty);
    if is_buy {
        msg.set(fix44::SIDE, fix44::Side::Buy);
    } else {
        msg.set(fix44::SIDE, fix44::Side::Sell);
    }
    msg.set(fix44::TIME_IN_FORCE, fix44::TimeInForce::Day);

    let a = msg.wrap();
    String::from_utf8(Vec::from(a)).unwrap()
}

fn build_cancel(is_buy: bool, order_id: u32) -> String {
    let mut encoder = Encoder::<Config>::default();
    encoder.config_mut().set_separator(b'|');
    let mut buffer = Vec::new();
    let mut msg = encoder.start_message(b"FIX.4.2", &mut buffer, b"F");
    msg.set(fix44::MSG_SEQ_NUM, 215);
    msg.set(fix44::SENDER_COMP_ID, "CLIENT12");
    msg.set(fix44::TARGET_COMP_ID, "B");
    msg.set(fix44::ACCOUNT, "TestClient");
    msg.set(fix44::ORDER_ID, order_id);
    msg.set(fix44::ORD_TYPE, fix44::OrdType::Limit);
    if is_buy {
        msg.set(fix44::SIDE, fix44::Side::Buy);
    } else {
        msg.set(fix44::SIDE, fix44::Side::Sell);
    }
    msg.set(fix44::TIME_IN_FORCE, fix44::TimeInForce::Day);

    let a = msg.wrap();
    String::from_utf8(Vec::from(a)).unwrap()
}