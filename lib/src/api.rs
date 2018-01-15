//! API for communicating between server and client

use std::io::{self, BufRead, BufReader, Read, Write};
use std::str::FromStr;

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

/// Argument kind from api
pub enum ApiArgKind {
    /// Target process
    Target,
    /// Other request
    Other(String),
}

impl<'a> From<&'a str> for ApiArgKind {
    fn from(s: &'a str) -> Self {
        ApiArgKind::from(s.to_owned())
    }
}

impl From<String> for ApiArgKind {
    fn from(s: String) -> Self {
        match s.as_ref() {
            "target" => ApiArgKind::Target,
            _ => ApiArgKind::Other(s),
        }
    }
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

    /// Get argument kind
    pub fn kind(&self) -> &ApiArgKind {
        &self.kind
    }

    /// Get argument value
    pub fn value(&self) -> &str {
        &self.val
    }
}

impl FromStr for ApiArg {
    type Error = ();

    fn from_str(s: &str) -> Result<Self, Self::Error> {
        let eq_sign = s.find('=');
        if eq_sign.is_none() {
            return Err(());
        }
    }
}

/// API request
pub struct ApiRequest {
    kind: ApiKind,
    args: Vec<ApiArg>,
}

impl ApiRequest {
    /// Send api request
    pub fn send<S: Read + Write>(self) -> io::Result<()> {}
}

/// Request builder
pub struct ApiRequestBuilder {
    req: ApiRequest,
}

impl ApiRequestBuilder {
    /// Create request builder
    pub fn new(kind: ApiKind) -> ApiRequestBuilder {
        ApiRequestBuilder {
            req: ApiRequest {
                kind: kind,
                args: Vec::new(),
            },
        }
    }

    /// Set kind
    pub fn kind(mut self, kind: ApiKind) -> ApiRequestBuilder {
        self.req.kind = Some(kind);
        self
    }

    /// Add arg
    pub fn arg(mut self, kind: ApiArgKind, val: String) -> ApiRequestBuilder {
        self.req.args.push(ApiArg::new(kind, val));
        self
    }

    /// Compute
    pub fn build(self) -> ApiRequest {
        self.req
    }
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
