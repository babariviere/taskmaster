#[macro_use]
extern crate taskmaster;

use std::env;
use std::io::{stdin, stdout, BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use taskmaster::api;
use taskmaster::config::*;
use taskmaster::log::*;

fn main() {
    init_logger(|logger| {
        logger.add_output(Output::stdout(
            LevelFilter::Blather,
            Some(Box::new(|log| {
                format!("[{}] {}", log.level(), log.message())
            })),
        ));
    });
    let mut f = ::std::fs::File::open("/Users/briviere/projects/taskmaster/sample.ini").unwrap();
    let mut buf = String::new();
    f.read_to_string(&mut buf).unwrap();
    let config = ConfigParser::new(&buf).parse();
    let mut stream = TcpStream::connect(("127.0.0.1", taskmaster::DEFAULT_PORT))
        .expect("unable to connect to server");
    info!("connected to {}", stream.peer_addr().unwrap());
    let prompt = config
        .ctl()
        .map(|c| c.prompt.to_owned())
        .unwrap_or("taskmaster> ".to_owned());
    let mut reader = BufReader::new(stdin());
    loop {
        print!("{}", prompt);
        stdout().flush().unwrap();
        let mut buf = String::new();
        reader.read_line(&mut buf).unwrap();
        if buf.len() == 0 {
            break;
        }
        match buf.trim() {
            "shutdown" => {
                api::send_data(&mut stream, b"shutdown").unwrap();
            }
            "wave" => {
                api::send_data(&mut stream, b"wave").unwrap();
            }
            "status" => {
                api::send_data(&mut stream, b"status").unwrap();
                let data = api::recv_data(&mut stream).unwrap();
                println!("{}", data.trim());
            }
            s => api::send_data(&mut stream, s).unwrap(),
        }
    }
}
