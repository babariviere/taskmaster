#[macro_use]
extern crate taskmaster;

use std::env;
use std::io::{stdin, stdout, BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use taskmaster::config::*;
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
            "exit" => {
                stream.write(&[0xde, 0xad, 0xbe, 0xef]).unwrap();
            }
            "wave" => {
                stream.write(&[0xca, 0xfe, 0xba, 0xbe]).unwrap();
            }
            "status" => {
                stream.write(&[0xaa, 0xaa, 0xaa, 0xaa]).unwrap();
                let mut stream = BufReader::new(&mut stream);
                loop {
                    let mut buf = String::new();
                    stream.read_line(&mut buf).unwrap();
                    let trimmed = buf.trim();
                    if trimmed == "end" {
                        break;
                    }
                    println!("{}", trimmed);
                }
            }
            _ => {}
        }
    }
}
