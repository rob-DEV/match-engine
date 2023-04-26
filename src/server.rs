use std::io::prelude::*;
use std::io::Result;
use std::net::TcpListener;
use std::net::TcpStream;

use crate::util;

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 512];
    stream.read(&mut buffer).unwrap();
    println!(
        "connection_request@{} -> {}",
        util::current_epoch_time(),
        String::from_utf8_lossy(&buffer[..])
    );
}

pub fn create_listener() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:2345")?;
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        handle_connection(stream);
    }
    Ok(())
}
