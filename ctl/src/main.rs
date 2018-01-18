#[macro_use]
extern crate taskmaster;

use std::io::{stdin, stdout, BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use taskmaster::api::{self, ApiKind, ApiRequestBuilder};
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
    let prompt = format!(
        "{}> ",
        config
            .ctl()
            .map(|c| c.prompt.to_owned())
            .unwrap_or("taskmaster".to_owned())
    );
    let mut reader = BufReader::new(stdin());
    loop {
        print!("{}", prompt);
        stdout().flush().unwrap();
        let mut buf = String::new();
        reader.read_line(&mut buf).unwrap();
        if buf.len() == 0 {
            break;
        }
        if buf.chars().next() == Some('\n') {
            continue;
        }
        match buf.trim() {
            "shutdown" => {
                ApiRequestBuilder::new(ApiKind::Shutdown)
                    .build()
                    .send(&mut stream)
                    .unwrap();
            }
            "status" => {
                ApiRequestBuilder::new(ApiKind::Status)
                    .build()
                    .send(&mut stream)
                    .unwrap();
                let data = String::from_utf8(api::recv_data(&mut stream).unwrap()).unwrap();
                println!("{}", data.trim());
            }
            "log" => {
                ApiRequestBuilder::new(ApiKind::Log)
                    .build()
                    .send(&mut stream)
                    .unwrap();
                let data = String::from_utf8(api::recv_data(&mut stream).unwrap()).unwrap();
                println!("{}", data.trim());
            }
            "kill" => {
                ApiRequestBuilder::new(ApiKind::Kill)
                    .build()
                    .send(&mut stream)
                    .unwrap();
            }
            "exit" => break,
            //s => api::send_data(&mut stream, s).unwrap(),
            _ => {
                error!("c'est pas valide ca monsieur");
            }
        }
    }
}
