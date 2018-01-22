//! Module to handle client

use ProcessSync;

use process::Process;
use std::io;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, RwLockReadGuard};
use std::thread;
use std::time::Duration;
use taskmaster::api::*;

pub fn handle_fg<'a>(stream: &mut TcpStream, process: RwLockReadGuard<'a, Process>) {
    let listener_stdin = TcpListener::bind("127.0.0.1:0").unwrap();
    let stdin_addr = listener_stdin.local_addr().unwrap();
    let listener_stdout = TcpListener::bind("127.0.0.1:0").unwrap();
    let stdout_addr = listener_stdout.local_addr().unwrap();
    send_data(stream, stdin_addr.port().to_string().as_bytes()).unwrap();
    send_data(stream, stdout_addr.port().to_string().as_bytes()).unwrap();
    let (mut stream_in, _) = listener_stdin.accept().unwrap();
    let (mut stream_out, _) = listener_stdout.accept().unwrap();
    stream_in.set_nonblocking(true).unwrap();
    stream_out.set_nonblocking(true).unwrap();
    loop {
        match recv_data(&mut stream_in) {
            Ok(s) => {
                blather!("received stdin");
                process.holder().write_stdin(&s).unwrap();
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
            Err(ref e) => {
                warn!("{}", e);
                break;
            }
        }
        let readed = process.holder().read_stdout();
        if readed.len() > 0 {
            match send_data(&mut stream_out, &readed[0..readed.len()]) {
                Ok(_) => {}
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
                Err(e) => {
                    warn!("{}", e);
                    break;
                }
            }
        }
        thread::sleep(Duration::from_millis(10));
    }
}

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
                send_data(&mut stream, data.as_bytes()).unwrap();
            }
            &ApiKind::Kill => {
                info!("kill request from {}", addr);
                for process in processes.iter() {
                    process.read().unwrap().kill();
                }
            }
            &ApiKind::Log => {
                let mut data = Vec::new();
                for process in processes.iter() {
                    let process = match process.read() {
                        Ok(p) => p,
                        Err(e) => {
                            error!("{:#?}", e);
                            continue;
                        }
                    };
                    let mut holder = process.holder();
                    holder.read_stdout();
                    let readed = holder.get_stdout();
                    data.extend(readed.iter());
                }
                send_data(&mut stream, &data).unwrap();
            }
            &ApiKind::Foreground => {
                let args = req.args();
                let mut target = None;
                for arg in args {
                    match arg.kind() {
                        &ApiArgKind::Target => target = Some(arg.value()),
                    }
                }
                match target {
                    Some(val) => {
                        let mut handled = false;
                        for process in processes.iter() {
                            let prc = process.read().unwrap();
                            if prc.proc_name() == val {
                                info!("foreground request from {}", addr);
                                handle_fg(&mut stream, prc);
                                info!("ended foreground from {}", addr);
                                handled = true;
                            }
                        }
                        if !handled {
                            send_data(
                                &mut stream,
                                format!("target {} does not exists", val).as_bytes(),
                            ).unwrap();
                        }
                    }
                    None => {
                        send_data(&mut stream, b"missing target option").unwrap();
                    }
                }
            }
            a => {
                warn!("unimplemented request `{}` from {}", a, addr);
                send_data(&mut stream, b"unimplemented").unwrap();
            }
        }
    }
}
