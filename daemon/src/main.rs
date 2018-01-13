#![feature(nll)]

extern crate nix;
#[macro_use]
extern crate taskmaster;

mod command;
mod process;

use nix::sys::stat::*;
use nix::unistd::*;
use process::*;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::process::exit;
use std::sync::{Arc, Mutex};
use std::thread;
use taskmaster::config::{Config, ConfigParser};
use taskmaster::ffi::close_all_fd;
use taskmaster::log::*;

fn daemonize() {
    match fork() {
        Ok(ForkResult::Parent { child }) => {
            info!("running on pid {}", child);
            exit(0);
        }
        Ok(ForkResult::Child) => {}
        Err(_) => exit(1),
    }
    chdir("/").unwrap();
    umask(Mode::empty());
    close_all_fd();
}

fn get_config() -> Config {
    let mut f = ::std::fs::File::open("/Users/briviere/projects/taskmaster/sample.ini").unwrap();
    let mut buf = String::new();
    f.read_to_string(&mut buf).unwrap();
    init_logger(|logger| {
        logger.add_output(Output::stdout(
            LevelFilter::Trace,
            Some(Box::new(|log| {
                format!("[{}] {}", log.level(), log.message())
            })),
        ));
    });
    ConfigParser::new(&buf).parse()
}

fn main() {
    let log_path = ::std::env::current_dir().unwrap().join("log");
    let config = get_config();
    trace!("config:\n{:#?}", config);
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
    let mut processes = Vec::new();
    for process in config.processes() {
        let p = process.clone();
        processes.push(Arc::new(Mutex::new(Process::new(p))));
    }
    blather!("spawning processes");
    // TODO: use threading from rust with Arc and Mutex
    for process in &processes {
        let process = process.clone();
        thread::spawn(move || {
            process.lock().unwrap().spawn();
        });
    }
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
                    (0xaa, 0xaa, 0xaa, 0xaa) => {
                        info!("status request from {}", stream.peer_addr().unwrap());
                        for process in &processes {
                            let mut process = process.lock().unwrap();
                            let name = process.proc_name().to_owned();
                            let state = process.get_state();
                            let status = format!("{} {:?}\n", name, state);
                            stream.write(status.as_bytes()).unwrap();
                        }
                        stream.write(b"end\n").unwrap();
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
