extern crate taskmaster;

use std::fs::File;
use std::io::Write;
use std::net::TcpListener;
use std::process::exit;
use taskmaster::ffi::{chdir, close_all_fd, umask};
use taskmaster::process::*;

fn main() {
    match fork() {
        Ok(ForkResult::Parent(pid)) => {
            println!("running on pid {}", pid);
            exit(0);
        }
        Ok(ForkResult::Child) => {}
        Err(_) => exit(1),
    }
    unsafe {
        chdir("/".as_ptr());
        umask(0);
        close_all_fd();
    }
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
            Ok(stream) => {
                writeln!(file, "connected with {}", stream.peer_addr().unwrap()).unwrap();
            }
            Err(e) => {
                writeln!(file, "connection failed {}", e).unwrap();
            }
        }
    }
    writeln!(file, "exiting").unwrap();
}
