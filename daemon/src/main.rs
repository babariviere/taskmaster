#![feature(libc)]

#[macro_use]
extern crate taskmaster;

use std::io::Read;
use std::net::TcpListener;
use std::process::exit;
use taskmaster::config::IniValue;
use taskmaster::ffi::close_all_fd;
use taskmaster::log::*;
use taskmaster::libc;
use taskmaster::process::*;

fn daemonize() {
    match fork() {
        Ok(ForkResult::Parent(pid)) => {
            println!("running on pid {}", pid);
            exit(0);
        }
        Ok(ForkResult::Child) => {}
        Err(_) => exit(1),
    }
    unsafe {
        libc::chdir("/".as_ptr() as *const _);
        libc::umask(0);
        close_all_fd();
    }
}

fn print_values(inis: &Vec<IniValue>) {
    for value in inis {
        match value {
            &IniValue::Key(ref n, ref v) => println!("{}: {}", n, v),
            &IniValue::Section(ref n, ref v) => {
                println!("section {}", n);
                print_values(v);
                println!();
            }
        }
    }
}

fn main() {
    let mut f = ::std::fs::File::open("/Users/briviere/projects/taskmaster/sample.ini").unwrap();
    let mut buf = String::new();
    f.read_to_string(&mut buf);
    let values = ::taskmaster::config::IniParser::new(&buf).parse();
    print_values(&values);
    let log_path = ::std::env::current_dir().unwrap().join("log");
    daemonize();
    init_logger(move |logger| {
        logger.add_output(
            Output::file(
                log_path,
                LevelFilter::Blather,
                Some(Box::new(|log| {
                    if log.level() as u8 >= LevelFilter::Debug as u8 {
                        format!(
                            "[{}] {}::{} {}",
                            log.level(),
                            log.file(),
                            log.line(),
                            log.message()
                        )
                    } else {
                        format!("[{}] {}", log.level(), log.message())
                    }
                })),
            ).unwrap(),
        )
    });
    info!("starting listener");
    let listener = match TcpListener::bind(("127.0.0.1", taskmaster::DEFAULT_PORT)) {
        Ok(l) => l,
        Err(e) => {
            error!("Error {}", e);
            exit(1);
        }
    };
    info!("listening");
    debug!("listener has start on port {}", taskmaster::DEFAULT_PORT);
    for client in listener.incoming() {
        match client {
            Ok(mut stream) => {
                info!("connected with {}", stream.peer_addr().unwrap());
                let mut buf = [0; 4];
                stream.read(&mut buf).unwrap();
                match (buf[0], buf[1], buf[2], buf[3]) {
                    (0xde, 0xad, 0xbe, 0xef) => {
                        warn!("exit instruction from {}", stream.peer_addr().unwrap());
                        break;
                    }
                    (0xca, 0xfe, 0xba, 0xbe) => {
                        info!("wave from {}", stream.peer_addr().unwrap());
                    }
                    _ => {}
                }
            }
            Err(e) => {
                error!("connection failed {}", e);
            }
        }
    }
    warn!("exiting");
}
