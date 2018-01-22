extern crate nix;
#[macro_use]
extern crate taskmaster;

use nix::fcntl;
use std::io::{stdin, stdout, BufRead, Read, Write};
use std::net::TcpStream;
use std::thread;
use std::time::Duration;
use taskmaster::api::{self, ApiArgKind, ApiKind, ApiRequestBuilder};
use taskmaster::config::*;
use taskmaster::log::*;

fn handle_fg(stream: &mut TcpStream) {
    let stdin_port = String::from_utf8(api::recv_data(stream).unwrap())
        .unwrap()
        .parse()
        .unwrap();
    let mut stdin_stream = match TcpStream::connect(("127.0.0.1", stdin_port)) {
        Ok(s) => s,
        Err(e) => {
            warn!("{}", e);
            return;
        }
    };
    let stdout_port = String::from_utf8(api::recv_data(stream).unwrap())
        .unwrap()
        .parse()
        .unwrap();
    let mut stdout_stream = match TcpStream::connect(("127.0.0.1", stdout_port)) {
        Ok(s) => s,
        Err(e) => {
            warn!("{}", e);
            return;
        }
    };
    stdout_stream.set_nonblocking(true).unwrap();
    loop {
        let mut buf = [0; 512];
        match stdin().read(&mut buf) {
            Ok(_) => {
                api::send_data(&mut stdin_stream, &buf).unwrap();
            }
            Err(ref e) if e.kind() == ::std::io::ErrorKind::WouldBlock => {}
            Err(e) => {
                error!("{:?}", e);
                break;
            }
        }
        //blather!("data: {:#?}", buf);
        match stdout_stream.read(&mut buf) {
            Ok(_) => {
                print!("{}", ::std::str::from_utf8(&buf).unwrap());
            }
            Err(ref e) if e.kind() == ::std::io::ErrorKind::WouldBlock => {}
            Err(e) => {
                error!("{:#?}", e);
                break;
            }
        }
        thread::sleep(Duration::from_millis(10));
    }
}

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
    loop {
        print!("{}", prompt);
        stdout().flush().unwrap();
        let mut buf = String::new();
        {
            let stdin = stdin();
            let mut lock = stdin.lock();
            lock.read_line(&mut buf).unwrap();
        }
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
            "fg" => {
                match fcntl::fcntl(0, fcntl::FcntlArg::F_GETFL) {
                    Ok(f) => {
                        let _ = fcntl::fcntl(
                            0,
                            fcntl::FcntlArg::F_SETFL(
                                fcntl::OFlag::from_bits_truncate(f) | fcntl::O_NONBLOCK,
                            ),
                        ).unwrap();
                    }
                    _ => {}
                }
                ApiRequestBuilder::new(ApiKind::Foreground)
                    .arg(ApiArgKind::Target, "theprogramname".to_owned())
                    .build()
                    .send(&mut stream)
                    .unwrap();
                handle_fg(&mut stream);
                match fcntl::fcntl(0, fcntl::FcntlArg::F_GETFL) {
                    Ok(f) => {
                        let _ = fcntl::fcntl(
                            0,
                            fcntl::FcntlArg::F_SETFL(
                                fcntl::OFlag::from_bits_truncate(f) & !fcntl::O_NONBLOCK,
                            ),
                        ).unwrap();
                    }
                    _ => {}
                }
            }
            "exit" => break,
            //s => api::send_data(&mut stream, s).unwrap(),
            _ => {
                error!("c'est pas valide ca monsieur");
            }
        }
    }
}
