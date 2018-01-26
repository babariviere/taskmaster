#[macro_use]
extern crate failure;
extern crate nix;
extern crate serde_yaml;
extern crate signal_notify as sig;
#[macro_use]
extern crate taskmaster;

use failure::Error;
use sig::Signal;
use std::io::{self, stdin, stdout, BufRead, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use taskmaster::api::{self, ApiArg, ApiArgKind, ApiKind, ApiRequestBuilder};
use taskmaster::config::*;
use taskmaster::log::*;

fn handle_fg(stream: &mut TcpStream, sign_recv: &mpsc::Receiver<Signal>) -> Result<(), Error> {
    let stdin_port = String::from_utf8(api::recv_data(stream)?)?.parse()?;
    let mut stdin_stream = match TcpStream::connect(("127.0.0.1", stdin_port)) {
        Ok(s) => s,
        Err(e) => {
            warn!("{}", e);
            bail!(e);
        }
    };
    let stdout_port = String::from_utf8(api::recv_data(stream)?)?.parse()?;
    let mut stdout_stream = match TcpStream::connect(("127.0.0.1", stdout_port)) {
        Ok(s) => s,
        Err(e) => {
            warn!("{}", e);
            bail!(e);
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
            Ok(sz) => {
                api::send_data(&mut stdin_stream, &buf[0..sz]).unwrap();
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
    Ok(())
}

#[derive(Debug, PartialEq)]
enum ParsingState {
    Normal,
    Buf,
    Quote(char),
}

fn parse_cli(buf: &str) -> Vec<String> {
    let mut res = Vec::new();
    let mut state = ParsingState::Normal;
    let mut curr = String::new();
    let mut esc = false;

    for c in buf.chars() {
        match c {
            '\\' => esc = true,
            '\'' | '\"' => {
                if state == ParsingState::Normal {
                    state = ParsingState::Quote(c);
                } else if esc {
                    curr.push(c);
                    esc = false;
                } else {
                    if let ParsingState::Quote(quote) = state {
                        if quote == c {
                            res.push(curr);
                            curr = String::new();
                            state = ParsingState::Normal;
                        } else {
                            curr.push(c);
                        }
                    }
                }
            }
            c if c.is_whitespace() => {
                if state == ParsingState::Normal {
                    continue;
                } else if state == ParsingState::Buf {
                    res.push(curr);
                    curr = String::new();
                    state = ParsingState::Normal;
                } else {
                    curr.push(c);
                }
            }
            c => {
                if state == ParsingState::Normal {
                    state = ParsingState::Buf;
                }
                curr.push(c);
            }
        }
    }
    if !curr.is_empty() {
        res.push(curr);
    }
    res
}

fn main_loop(
    mut stream: TcpStream,
    sign_recv: mpsc::Receiver<Signal>,
    prompt: String,
) -> Result<(), Error> {
    // TODO: Ctrl C in main
    loop {
        if let Ok(_sig) = sign_recv.try_recv() {
            print!("\x21[2K\r");
        }
        print!("{}", prompt);
        stdout().flush()?;
        let mut buf = String::new();
        {
            let stdin = stdin();
            let mut lock = stdin.lock();
            lock.read_line(&mut buf)?;
        }
        if buf.len() == 0 {
            break;
        }
        if buf.chars().next() == Some('\n') {
            continue;
        }
        let parsed = parse_cli(&buf);
        match parsed[0].as_ref() {
            "shutdown" => {
                ApiRequestBuilder::new(ApiKind::Shutdown)
                    .build()
                    .send(&mut stream)?;
            }
            "status" => {
                ApiRequestBuilder::new(ApiKind::Status)
                    .build()
                    .send(&mut stream)?;
                let data = String::from_utf8(api::recv_data(&mut stream)?)?;
                println!("{}", data.trim());
            }
            "log" => {
                if parsed.len() != 2 {
                    error!("invalid cli command");
                }
                ApiRequestBuilder::new(ApiKind::Log)
                    .arg(ApiArgKind::Target, parsed[1].to_owned())
                    .build()
                    .send(&mut stream)?;
                let data = String::from_utf8(api::recv_data(&mut stream)?)?;
                println!("{}", data.trim());
            }
            "kill" => {
                ApiRequestBuilder::new(ApiKind::Kill)
                    .args(
                        parsed[1..]
                            .iter()
                            .map(|a| ApiArg::new(ApiArgKind::Target, a.to_owned()))
                            .collect(),
                    )
                    .build()
                    .send(&mut stream)?;
            }
            "fg" => {
                if parsed.len() != 2 {
                    error!("invalid cli command");
                }
                ApiRequestBuilder::new(ApiKind::Foreground)
                    .arg(ApiArgKind::Target, parsed[1].to_owned())
                    .build()
                    .send(&mut stream)?;
                match handle_fg(&mut stream, &sign_recv) {
                    Ok(()) => {}
                    Err(e) => {
                        warn!("error in fg: {}", e);
                    }
                }
            }
            "exit" => break,
            _ => {
                error!("c'est pas valide ca monsieur");
            }
        }
    }
    Ok(())
}

fn main_wrapper() -> Result<(), Error> {
    init_logger(|logger| {
        logger.add_output(Output::stdout(
            LevelFilter::Blather,
            Some(Box::new(|log| {
                format!(
                    "{}:{} [{}] {}",
                    log.file(),
                    log.line(),
                    log.level().colored(),
                    log.message()
                )
            })),
        ));
    });
    let sign_recv = sig::notify(&[Signal::INT]);
    let mut f = ::std::fs::File::open("/Users/briviere/projects/taskmaster/sample.ini")?;
    let mut buf = String::new();
    f.read_to_string(&mut buf)?;
    let config: Config = ConfigParser::new(buf).parse();
    println!("{}", serde_yaml::to_string(&config).unwrap());
    let stream = TcpStream::connect(("127.0.0.1", taskmaster::DEFAULT_PORT))?;
    info!("connected to {}", stream.peer_addr()?);
    let prompt = format!(
        "{}> ",
        config
            .ctl()
            .map(|c| c.prompt.to_owned())
            .unwrap_or("taskmaster".to_owned())
    );
    match main_loop(stream, sign_recv, prompt) {
        Ok(()) => {}
        Err(e) => {
            trace!("{}", e);
        }
    };
    Ok(())
}

fn main() {
    let mut f = ::std::fs::File::open("/Users/briviere/projects/taskmaster/ctl/test.yml").unwrap();
    let config: Config = serde_yaml::from_reader(&mut f).unwrap();
    println!("{:#?}", config);
    match main_wrapper() {
        Ok(()) => {}
        Err(e) => {
            trace!("{}", e);
            if ::std::env::var("RUST_BACKTRACE") == Ok("1".to_owned()) {
                println!("{}", e.backtrace());
            }
        }
    }
}
