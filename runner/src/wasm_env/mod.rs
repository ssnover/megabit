use std::{cell::RefCell, collections::BTreeMap, path::Path, rc::Rc};

use crate::serial::SyncSerialConnection;

mod host_functions;

const SCREEN_WIDTH: usize = 32;
const SCREEN_HEIGHT: usize = 16;

pub type ScreenBuffer = [[bool; SCREEN_WIDTH]; SCREEN_HEIGHT];
pub type KvStore = BTreeMap<String, Vec<u8>>;

struct PersistentData {
    screen_buffer: Rc<RefCell<ScreenBuffer>>,
    kv_store: Rc<RefCell<KvStore>>,
    serial_conn: SyncSerialConnection,
}

pub struct WasmAppRunner {
    app: extism::Plugin,
}

impl WasmAppRunner {
    pub fn new(
        app_path: impl AsRef<Path>,
        serial_conn: SyncSerialConnection,
    ) -> anyhow::Result<Self> {
        let wasm_app_bin = extism::Wasm::file(app_path);
        let screen_buffer = Rc::new(RefCell::new([[false; SCREEN_WIDTH]; SCREEN_HEIGHT]));
        let kv_store = Rc::new(RefCell::new(BTreeMap::new()));

        let data = PersistentData {
            screen_buffer,
            kv_store,
            serial_conn,
        };
        let user_data = extism::UserData::new(data);

        let manifest = extism::Manifest::new([wasm_app_bin]);
        let plugin = extism::PluginBuilder::new(manifest)
            .with_wasi(true)
            .with_function(
                "write_region",
                [
                    extism::PTR,
                    extism::PTR,
                    extism::PTR,
                    extism::PTR,
                    extism::PTR,
                ],
                [extism::PTR],
                user_data.clone(),
                host_functions::write_region,
            )
            .with_function(
                "render",
                [extism::PTR],
                [extism::PTR],
                user_data.clone(),
                host_functions::render,
            )
            .with_function(
                "kv_store_read",
                [extism::PTR],
                [extism::PTR],
                user_data.clone(),
                host_functions::kv_store_read,
            )
            .with_function(
                "kv_store_write",
                [extism::PTR, extism::PTR],
                [extism::PTR],
                user_data.clone(),
                host_functions::kv_store_write,
            )
            .with_function(
                "log",
                [extism::PTR, extism::PTR],
                [extism::PTR],
                extism::UserData::new(()),
                host_functions::log,
            )
            .build()?;

        Ok(WasmAppRunner { app: plugin })
    }

    pub fn setup_app(&mut self) -> anyhow::Result<()> {
        self.app.call::<_, ()>("setup", ())
    }

    pub fn run_app_once(&mut self) -> anyhow::Result<()> {
        self.app.call::<_, ()>("run", ())
    }
}
