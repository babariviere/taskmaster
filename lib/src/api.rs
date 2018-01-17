//! API for communicating between server and client

use std::io::{self, BufRead, BufReader, Read, Write};
use std::fmt::{self, Display};
use std::str::FromStr;

macro_rules! impl_enum_str {
    (
        $(#[$attr:meta])*
        pub enum $target:ident {
            $(
                $(#[$doc:meta])*
                $e:ident => $s:tt
            ),*
        }
        $($extra:item)*
    ) => (
        $(#[$attr])*
        pub enum $target {
            $(
                $(#[$doc])*
                $e
            ),*
        }

        $($extra)*

        impl FromStr for $target {
            type Err = String;

            fn from_str(s: &str) -> Result<$target, String> {
                match s {
                    $(
                        $s => Ok($target::$e),
                    )*
                    _ => Err(s.to_owned()),
                }
            }
        }

        impl Display for $target {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                match self {
                    $(
                        &$target::$e => write!(f, $s),
                    )*
                }
            }
        }
    )
}

impl_enum_str! (
/// API request kind
#[derive(Debug, PartialEq)]
pub enum ApiKind {
    /// Request daemon log
    DaemonLog => "daemon_log",
    /// Request process log
    Log => "log",
    /// Request process status
    Status => "status",
    /// Request process kill
    Kill => "kill",
    /// Request process spawning
    Start => "start",
    /// Request process restart
    Restart => "restart",
    /// Request server shutdown
    Shutdown => "shutdown",
    /// Request server version
    Version => "version"
});

impl_enum_str! (
/// Argument kind from api
#[derive(Debug, PartialEq)]
pub enum ApiArgKind {
    /// Target process
    Target => "target"
});

/// API request argument
#[derive(Debug, PartialEq)]
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
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let eq_sign = s.find('=');
        if eq_sign.is_none() {
            return Err("no equal sign".to_owned());
        }
        let eq_sign = eq_sign.unwrap();
        Ok(ApiArg {
            kind: ApiArgKind::from_str(&s[0..eq_sign])?,
            val: s[eq_sign + 1..].to_string(),
        })
    }
}

impl Display for ApiArg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}={}", self.kind, self.val)
    }
}

/// API request
#[derive(Debug, PartialEq)]
pub struct ApiRequest {
    kind: ApiKind,
    args: Vec<ApiArg>,
}

impl ApiRequest {
    /// Send api request
    pub fn send<S: Read + Write>(self, stream: &mut S) -> io::Result<()> {
        let data = self.to_string();
        send_data(stream, data)
    }

    /// Recv api request
    pub fn recv<S: Read + Write>(stream: &mut S) -> Result<ApiRequest, String> {
        let data = match recv_data(stream) {
            Ok(d) => d,
            Err(_) => return Err("cannot receive data".to_owned()),
        };
        ApiRequest::from_str(&data)
    }

    /// Get api request kind
    pub fn kind(&self) -> &ApiKind {
        &self.kind
    }

    /// Get api request args
    pub fn args(&self) -> &Vec<ApiArg> {
        &self.args
    }
}

impl FromStr for ApiRequest {
    type Err = String;

    fn from_str(s: &str) -> Result<ApiRequest, Self::Err> {
        if s.chars().next() != Some('[') {
            return Err("missing bracket".to_owned());
        }
        let idx = match s.find(']') {
            Some(idx) => idx,
            None => return Err("missing bracket".to_owned()),
        };
        let kind = ApiKind::from_str(&s[1..idx]).map_err(|s| format!("unexpected value {}", s))?;
        if s.len() < (idx + 2) {
            return Err("missing arguments".to_owned());
        }
        let args = s[idx + 2..]
            .split(',')
            .filter_map(|s| ApiArg::from_str(s).ok())
            .collect();
        Ok(ApiRequest {
            kind: kind,
            args: args,
        })
    }
}

impl Display for ApiRequest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[{}] {}",
            self.kind.to_string(),
            self.args
                .iter()
                .map(|a| a.to_string())
                .collect::<Vec<String>>()
                .join(",")
        )
    }
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
        self.req.kind = kind;
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

#[cfg(test)]
mod unit_test {
    use super::*;

    #[test]
    fn test_api_request() {
        let req = ApiRequestBuilder::new(ApiKind::Version)
            .arg(ApiArgKind::Target, "appname".to_owned())
            .build();
        let req_str = req.to_string();
        let parsed_req = ApiRequest::from_str(&req_str).unwrap();
        assert_eq!(req, parsed_req);
    }
}
