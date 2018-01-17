//! Module to handle client

use ProcessSync;

use std::net::TcpStream;
use std::sync::Arc;
use taskmaster::api::*;

/// as it says, handle a client
pub fn handle_client(mut stream: TcpStream, processes: Arc<Vec<ProcessSync>>) {
    let addr = stream.peer_addr().unwrap();
    info!("connected with {}", addr);
    loop {
        let req = match ApiRequest::recv(&mut stream) {
            Ok(req) => req,
            Err(e) => {
                send_data(&mut stream, b"invalid request").unwrap();
                trace!("invalid request from {}: {}", addr, e);
                continue;
            }
        };
        trace!("request: {}", req);
        match req.kind() {
            &ApiKind::Shutdown => {
                warn!("shutdown instruction from {}", addr);
                break;
            }
            &ApiKind::Status => {
                info!("status request from {}", addr);
                let mut data = String::new();
                for process in processes.iter() {
                    let mut process = process.read().unwrap();
                    let name = process.proc_name().to_owned();
                    let state = process.get_state();
                    data.push_str(&format!("{} {:?}\n", name, *state));
                }
                send_data(&mut stream, data).unwrap();
            }
            &ApiKind::Kill => {
                info!("kill request from {}", addr);
                for process in processes.iter() {
                    process.read().unwrap().kill();
                }
            }
            a => {
                warn!("unimplemented request `{}` from {}", a, addr);
                send_data(&mut stream, b"unimplemented").unwrap();
            }
        }
    }
}
