pub mod display;
pub mod host;
pub mod kv_store;
pub mod log;
use display::DisplayConfiguration;
use extism_pdk::FnResult;
pub use megabit_wasm_macro::megabit_wasm_app;

pub trait MegabitApp {
    fn setup(display_cfg: DisplayConfiguration) -> FnResult<Self>
    where
        Self: Sized;

    fn run(&mut self) -> FnResult<()>;
}
