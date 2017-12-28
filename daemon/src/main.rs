#![feature(libc)]

extern crate taskmaster;

use std::fs::File;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::process::exit;
use taskmaster::ffi::close_all_fd;
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

fn main() {
    let mut file = File::create("log").unwrap();
    writeln!(file, "starting listener").unwrap();
    let listener = match TcpListener::bind(("127.0.0.1", taskmaster::DEFAULT_PORT)) {
        Ok(l) => l,
        Err(e) => {
            writeln!(file, "Error {}", e).unwrap();
            exit(1);
        }
    };
    writeln!(file, "listening").unwrap();
    for client in listener.incoming() {
        match client {
            Ok(mut stream) => {
                writeln!(file, "connected with {}", stream.peer_addr().unwrap()).unwrap();
                let mut buf = [0; 4];
                stream.read(&mut buf).unwrap();
                match (buf[0], buf[1], buf[2], buf[3]) {
                    (0xde, 0xad, 0xbe, 0xef) => {
                        writeln!(
                            file,
                            "exit instruction from {}",
                            stream.peer_addr().unwrap()
                        ).unwrap();
                        break;
                    }
                    (0xca, 0xfe, 0xba, 0xbe) => {
                        writeln!(file, "wave from {}", stream.peer_addr().unwrap()).unwrap();
                    }
                    _ => {}
                }
            }
            Err(e) => {
                writeln!(file, "connection failed {}", e).unwrap();
            }
        }
    }
    writeln!(file, "exiting").unwrap();
}
