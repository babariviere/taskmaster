extern crate taskmaster;

use std::net::TcpStream;

fn main() {
    let stream = TcpStream::connect(("127.0.0.1", taskmaster::DEFAULT_PORT)).expect("unable to connect to server");
    println!("connected to {}", stream.peer_addr().unwrap());
}
