pub use log::Level;

mod imports {
    use extism_pdk::*;

    #[host_fn]
    extern "ExtismHost" {
        pub fn log(level: u32, line: String) -> ();
    }
}

pub fn log(level: Level, line: impl Into<String>) {
    let level = match level {
        Level::Error => 4,
        Level::Warn => 3,
        Level::Info => 2,
        Level::Debug => 1,
        Level::Trace => 0,
    };
    let _ = unsafe { imports::log(level, line.into()) };
}
