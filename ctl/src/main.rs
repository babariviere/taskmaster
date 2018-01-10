#[macro_use]
extern crate taskmaster;

use std::env;
use std::io::Write;
use std::net::TcpStream;
use taskmaster::log::*;

fn main() {
    init_logger(|logger| {
        logger.add_output(Output::stdout(
            LevelFilter::Info,
            Some(Box::new(|log| {
                format!("[{}] {}", log.level(), log.message())
            })),
        ));
    });
    let mut args = env::args();
    args.next();
    let mut stream = TcpStream::connect(("127.0.0.1", taskmaster::DEFAULT_PORT))
        .expect("unable to connect to server");
    info!("connected to {}", stream.peer_addr().unwrap());
    match args.next() {
        Some(ref s) if s == "exit" => {
            stream.write(&[0xde, 0xad, 0xbe, 0xef]).unwrap();
        }
        Some(ref s) if s == "wave" => {
            stream.write(&[0xca, 0xfe, 0xba, 0xbe]).unwrap();
        }
        _ => {}
    }
}
