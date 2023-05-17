use std::{net::SocketAddr, fs::read};

use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    sync::broadcast,
};

mod engine;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        // A new task is spawned for each inbound socket. The socket is
        // moved to the new task and processed there.
        tokio::spawn(async move {
            process(socket).await;
        });
    }

    //client - send random orders every second
    // server receive order, sequence, add to book
}

async fn process(mut tcp_stream: TcpStream) {
    println!("Connection Established");

    let (reader, mut writer) = tcp_stream.split();

    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    while let Ok(bytes_read_successfully) = reader.read_line(&mut line).await {
        if bytes_read_successfully == 0 {
            break;
        }

        println!("Read input of: {bytes_read_successfully} bytes");
        println!("Input: {:?}", line.trim().as_bytes());

        line.clear();
        writer.write_all("Response\n".to_owned().as_bytes()).await.unwrap();  
    }

    println!("Connection closed!");
}
