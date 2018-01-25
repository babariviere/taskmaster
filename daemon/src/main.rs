#![feature(nll)]

extern crate failure;
extern crate nix;
extern crate serde_yaml;
#[macro_use]
extern crate taskmaster;

mod client;
mod command;
mod process;

use failure::Error;
use nix::sys::stat::*;
use nix::unistd::*;
use process::*;
use std::io::Read;
use std::net::TcpListener;
use std::process::exit;
use std::sync::{Arc, RwLock};
use std::thread;
use taskmaster::config::*;
use taskmaster::ffi::close_all_fd;
use taskmaster::log::*;

type ProcessSync = Arc<RwLock<Process>>;

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

fn get_config() -> Result<Config, Error> {
    let mut f = ::std::fs::File::open("/Users/briviere/projects/taskmaster/sample.yml")?;
    //let mut buf = String::new();
    //f.read_to_string(&mut buf)?;
    //Ok(ConfigParser::new(buf).parse())
    serde_yaml::from_reader(&mut f).map_err(|e| e.into())
}

fn main_wrapper() -> Result<(), Error> {
    let log_path = ::std::env::current_dir()?.join("log");
    let config = get_config()?;
    trace!("config:\n{:#?}", config);
    trace!("yaml: {}", serde_yaml::to_string(&config).unwrap());
    daemonize();
    init_logger(move |logger| {
        logger.add_output(
            Output::file(
                log_path,
                LevelFilter::Blather,
                Some(Box::new(|log| {
                    //if log.level() as u8 >= LevelFilter::Debug as u8 {
                    format!(
                        "[{}] {}::{} {}",
                        log.level(),
                        log.file(),
                        log.line(),
                        log.message()
                    )
                    // } else {
                    //     format!("[{}] {}", log.level(), log.message())
                    // }
                })),
            ).unwrap(),
        )
    });
    let mut processes = Vec::new();
    for process in config.processes() {
        let p = process.clone();
        processes.push(Arc::new(RwLock::new(Process::new(p))));
    }
    let processes = Arc::new(processes);
    blather!("spawning processes");
    for process in processes.iter() {
        let process = process.clone();
        thread::spawn(move || {
            process.write().unwrap().spawn();
            process.read().unwrap().track_state();
        });
    }
    info!("starting listener");
    let listener = TcpListener::bind(("127.0.0.1", taskmaster::DEFAULT_PORT))?;
    info!("listening");
    debug!("listener has start on port {}", taskmaster::DEFAULT_PORT);
    for client in listener.incoming() {
        match client {
            Ok(mut stream) => {
                let processes = processes.clone();
                thread::spawn(move || {
                    client::handle_client(stream, processes);
                });
            }
            Err(e) => {
                error!("connection failed {}", e);
            }
        }
    }
    warn!("exiting");
    Ok(())
}

fn main() {
    init_logger(|log| {
        log.add_output(Output::stdout(
            LevelFilter::Blather,
            Some(Box::new(|log| {
                format!("[{}] {}", log.level().colored(), log.message())
            })),
        ))
    });
    match main_wrapper() {
        Ok(()) => {}
        Err(e) => {
            error!("{}", e);
            if ::std::env::var("RUST_BACKTRACE") == Ok("1".to_owned()) {
                trace!("\n{}", e.backtrace());
            }
        }
    }
}
