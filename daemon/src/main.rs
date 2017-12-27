extern crate taskmaster;

use std::net::TcpListener;

fn main() {
    let listener = TcpListener::bind(("127.0.0.1", taskmaster::DEFAULT_PORT)).expect("unable to start listener");
    for client in listener.incoming() {
        match client {
            Ok(stream) => {
                println!("connected with {}", stream.peer_addr().unwrap());
            }
            Err(e) => {
                println!("connection failed {}", e);
            }
        }
    }
}
