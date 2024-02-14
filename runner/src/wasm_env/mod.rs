use self::host_functions::with_host_functions;
use crate::{
    display::{DisplayConfiguration, ScreenBuffer, DEFAULT_MONO_PALETTE},
    serial::SyncSerialConnection,
};
use app_manifest::AppManifest;
use std::{cell::RefCell, collections::BTreeMap, path::Path, rc::Rc};

mod app_manifest;
mod host_functions;

pub type KvStore = BTreeMap<String, Vec<u8>>;

struct PersistentData {
    screen_buffer: Rc<RefCell<ScreenBuffer>>,
    kv_store: Rc<RefCell<KvStore>>,
    serial_conn: SyncSerialConnection,
}

impl PersistentData {
    fn new(serial_conn: SyncSerialConnection, display_cfg: DisplayConfiguration) -> Self {
        let screen_buffer = Rc::new(RefCell::new(ScreenBuffer::new(
            display_cfg.width,
            display_cfg.height,
            if display_cfg.is_rgb {
                Some(DEFAULT_MONO_PALETTE)
            } else {
                None
            },
        )));
        let kv_store = Rc::new(RefCell::new(BTreeMap::new()));

        PersistentData {
            screen_buffer,
            kv_store,
            serial_conn,
        }
    }
}

pub struct WasmAppRunner {
    app: extism::Plugin,
    name: String,
}

impl WasmAppRunner {
    pub fn new(
        app_path: impl AsRef<Path>,
        serial_conn: SyncSerialConnection,
        display_cfg: DisplayConfiguration,
    ) -> anyhow::Result<Self> {
        let app_manifest = AppManifest::open(app_path)?;
        let wasm_app_bin = extism::Wasm::file(app_manifest.app_bin_path);
        let user_data = extism::UserData::new(PersistentData::new(serial_conn, display_cfg));
        let manifest = extism::Manifest::new([wasm_app_bin]);
        let plugin = with_host_functions(extism::PluginBuilder::new(manifest), &user_data)
            .with_wasi(true)
            .build()?;

        Ok(WasmAppRunner {
            app: plugin,
            name: app_manifest.app_name,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn setup_app(&mut self) -> anyhow::Result<()> {
        self.app.call::<_, ()>("setup", ())
    }

    pub fn run_app_once(&mut self) -> anyhow::Result<()> {
        self.app.call::<_, ()>("run", ())
    }
}
