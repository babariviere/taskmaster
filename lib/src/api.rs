//! API for communicating between server and client
use std::io::{self, BufRead, BufReader, Read, Write};

/// API request kind
pub enum ApiKind {
    /// Request daemon log
    DaemonLog,
    /// Request process log
    Log,
    /// Request process status
    Status,
    /// Request process kill
    Kill,
    /// Request process spawning
    Start,
    /// Request process restart
    Restart,
    /// Request server shutdown
    Shutdown,
    /// Request server version
    Version,
}

pub enum ApiArgKind {
    /// Target process
    Target,
    /// Other request
    Other(String),
}

/// API request argument
pub struct ApiArg {
    kind: ApiArgKind,
    val: String,
}

impl ApiArg {
    /// Create new arg
    pub fn new(kind: ApiArgKind, val: String) -> ApiArg {
        ApiArg {
            kind: kind,
            val: val,
        }
    }
}

/// API request
pub struct ApiRequest {
    kind: ApiKind,
    args: Vec<ApiArg>,
}

/// Send chunk of data
pub fn send_data<S: Read + Write, D: AsRef<[u8]>>(stream: &mut S, data: D) -> io::Result<()> {
    let data = data.as_ref();
    stream.write_all(data)?;
    if data[data.len() - 1] != b'\n' {
        stream.write(b"\n")?;
    }
    stream.write(b"end\n")?;
    blather!("data are sent");
    let mut buf = [0; 2];
    stream.read(&mut buf)?;
    assert_eq!(&buf, b"OK");
    blather!("received OK");
    Ok(())
}

/// Receive chunk of data
pub fn recv_data<S: Read + Write>(mut stream: &mut S) -> io::Result<String> {
    let mut buf = String::new();
    let mut line = String::new();
    {
        let mut stream = BufReader::new(&mut stream);
        loop {
            stream.read_line(&mut line)?;
            if line == "end\n" {
                break;
            }
            buf.push_str(&line);
            line = String::new();
        }
    }
    blather!("data readed");
    stream.write(b"OK")?;
    blather!("sent OK");
    Ok(buf)
}
