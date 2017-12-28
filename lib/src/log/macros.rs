#[macro_export]
macro_rules! log {
    ($lvl:expr, $($arg:tt)+) => ({
        $crate::log::logger().log(
            $crate::log::LogBuilder::new()
                    .message(&format!("{}", format_args!($($arg)+)))
                    .level($lvl)
                    .module(module_path!())
                    .file(file!())
                    .line(line!())
                    .build()
            )
    })
}

#[macro_export]
macro_rules! critical {
    ($($arg:tt)+) => (
        log!($crate::log::Level::Critical, $($arg)+)
    )
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)+) => (
        log!($crate::log::Level::Error, $($arg)+)
    )
}
#[macro_export]
macro_rules! warn {
    ($($arg:tt)+) => (
        log!($crate::log::Level::Warn, $($arg)+)
    )
}
#[macro_export]
macro_rules! info {
    ($($arg:tt)+) => (
        log!($crate::log::Level::Info, $($arg)+)
    )
}
#[macro_export]
macro_rules! debug {
    ($($arg:tt)+) => (
        log!($crate::log::Level::Debug, $($arg)+)
    )
}
#[macro_export]
macro_rules! trace {
    ($($arg:tt)+) => (
        log!($crate::log::Level::Trace, $($arg)+)
    )
}

#[macro_export]
macro_rules! blather {
    ($($arg:tt)+) => (
        log!($crate::log::Level::Blather, $($arg)+)
    )
}
