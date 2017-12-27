#[repr(u8)]
#[derive(Debug)]
pub enum LogLevel {
    Critical,
    Error,
    Warn,
    Info,
    Debug,
    Blather,
}
