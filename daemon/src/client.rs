//! Module to handle client

use ProcessSync;

use std::net::TcpStream;
use std::sync::Arc;
use taskmaster::api;

/// as it says, handle a client
pub fn handle_client(mut stream: TcpStream, processes: Arc<Vec<ProcessSync>>) {
    let addr = stream.peer_addr().unwrap();
    info!("connected with {}", addr);
    loop {
        let recv = api::recv_data(&mut stream).unwrap();
        match recv.trim() {
            "shutdown" => {
                warn!("shutdown instruction from {}", addr);
                break;
            }
            "wave" => {
                info!("wave from {}", addr);
            }
            "status" => {
                info!("status request from {}", addr);
                let mut data = String::new();
                for process in processes.iter() {
                    let mut process = process.read().unwrap();
                    let name = process.proc_name().to_owned();
                    let state = process.get_state();
                    data.push_str(&format!("{} {:?}\n", name, *state));
                }
                api::send_data(&mut stream, data).unwrap();
            }
            "kill" => {
                info!("kill request from {}", addr);
                for process in processes.iter() {
                    process.read().unwrap().kill();
                }
            }
            s => warn!("unknown request `{}` from {}", s, addr),
        }
    }
}
