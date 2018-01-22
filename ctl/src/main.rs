extern crate nix;
extern crate signal_notify as sig;
#[macro_use]
extern crate taskmaster;

use nix::fcntl;
use sig::Signal;
use std::io::{self, stdin, stdout, BufRead, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use taskmaster::api::{self, ApiArgKind, ApiKind, ApiRequestBuilder};
use taskmaster::config::*;
use taskmaster::log::*;

fn handle_fg(stream: &mut TcpStream, sign_recv: &mpsc::Receiver<Signal>) {
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
    thread::spawn(move || loop {
        let mut buf = [0; 512];
        match stdout_stream.read(&mut buf) {
            Ok(sz) => {
                print!("{}", ::std::str::from_utf8(&buf[0..sz]).unwrap());
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
            Err(e) => {
                error!("{:#?}", e);
                return;
            }
        }
        thread::sleep(Duration::from_millis(10));
    });
    loop {
        let mut buf = [0; 512];
        match stdin().read(&mut buf) {
            Ok(_) => {
                api::send_data(&mut stdin_stream, &buf).unwrap();
            }
            Err(ref e) if e.kind() == io::ErrorKind::Interrupted => break,
            Err(e) => {
                error!("{:?}", e);
                break;
            }
        }
        if let Ok(sig) = sign_recv.try_recv() {
            print!("\r");
            if sig == Signal::INT {
                break;
            }
        }
    }
}

fn main() {
    init_logger(|logger| {
        logger.add_output(Output::stdout(
            LevelFilter::Info,
            Some(Box::new(|log| {
                format!(
                    "{}:{} [{}] {}",
                    log.file(),
                    log.line(),
                    log.level(),
                    log.message()
                )
            })),
        ));
    });
    let sign_recv = sig::notify(&[Signal::INT]);
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
        if let Ok(_sig) = sign_recv.try_recv() {
            print!("\x21[2K\r");
        }
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
                //match fcntl::fcntl(0, fcntl::FcntlArg::F_GETFL) {
                //    Ok(f) => {
                //        let _ = fcntl::fcntl(
                //            0,
                //            fcntl::FcntlArg::F_SETFL(
                //                fcntl::OFlag::from_bits_truncate(f) | fcntl::O_NONBLOCK,
                //            ),
                //        ).unwrap();
                //    }
                //    _ => {}
                //}
                ApiRequestBuilder::new(ApiKind::Foreground)
                    .arg(ApiArgKind::Target, "theprogramname".to_owned())
                    .build()
                    .send(&mut stream)
                    .unwrap();
                handle_fg(&mut stream, &sign_recv);
                //match fcntl::fcntl(0, fcntl::FcntlArg::F_GETFL) {
                //    Ok(f) => {
                //        let _ = fcntl::fcntl(
                //            0,
                //            fcntl::FcntlArg::F_SETFL(
                //                fcntl::OFlag::from_bits_truncate(f) & !fcntl::O_NONBLOCK,
                //            ),
                //        ).unwrap();
                //    }
                //    _ => {}
                //}
            }
            "exit" => break,
            //s => api::send_data(&mut stream, s).unwrap(),
            _ => {
                error!("c'est pas valide ca monsieur");
            }
        }
    }
}
