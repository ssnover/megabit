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
                "set_monocolor_palette",
                [extism::PTR, extism::PTR],
                [extism::PTR],
                user_data.clone(),
                host_functions::set_monocolor_palette,
            )
            .with_function(
                "get_display_info",
                [],
                [extism::PTR],
                user_data.clone(),
                host_functions::get_display_info,
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
