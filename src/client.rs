use std::io::prelude::*;
use std::io::Result;
use std::net::TcpStream;

pub fn connect() -> Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:2345")?;
    let msg = b"INPUT";
    stream.write(msg)?;
    stream.read(&mut [0; 128])?;
    Ok(())
}