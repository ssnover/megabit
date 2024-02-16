use crate::host;
pub use log::Level;

pub fn log(level: Level, line: impl Into<String>) {
    let level = match level {
        Level::Error => 4,
        Level::Warn => 3,
        Level::Info => 2,
        Level::Debug => 1,
        Level::Trace => 0,
    };
    let _ = unsafe { host::log(level, line.into()) };
}
