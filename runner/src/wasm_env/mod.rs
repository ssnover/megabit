use self::host_functions::with_host_functions;
use crate::{
    display::{DisplayConfiguration, ScreenBuffer},
    transport::SyncConnection,
};
pub use app_manifest::AppManifest;
use std::{cell::RefCell, collections::BTreeMap, io, path::Path, rc::Rc, time::Duration};

mod app_manifest;
mod host_functions;

pub type KvStore = BTreeMap<String, Vec<u8>>;

struct PersistentData {
    screen_buffer: Rc<RefCell<ScreenBuffer>>,
    kv_store: Rc<RefCell<KvStore>>,
    serial_conn: SyncConnection,
}

impl PersistentData {
    fn new(serial_conn: SyncConnection, display_cfg: DisplayConfiguration) -> Self {
        let screen_buffer = Rc::new(RefCell::new(ScreenBuffer::new(
            display_cfg.width,
            display_cfg.height,
            display_cfg.is_rgb,
            None,
        )));
        let kv_store = Rc::new(RefCell::new(BTreeMap::new()));

        PersistentData {
            screen_buffer,
            kv_store,
            serial_conn,
        }
    }
}

pub fn load_apps_from_path(data_dir: impl AsRef<Path>) -> io::Result<Vec<AppManifest>> {
    Ok(std::fs::read_dir(data_dir)?
        .map(|entry| {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                if let Ok(app) = AppManifest::open(&path) {
                    tracing::info!("Found app {} at path: {}", app.app_name, app.path.display());
                    Ok(Some(app))
                } else {
                    tracing::warn!("Unable to build app from bundle at {}", path.display());
                    Ok(None)
                }
            } else {
                Ok(None)
            }
        })
        .filter_map(|app: anyhow::Result<Option<AppManifest>>| match app {
            Ok(app) => app,
            Err(_) => None,
        })
        .collect::<Vec<_>>())
}

pub struct WasmAppRunner {
    plugin: extism::Plugin,
    name: String,
    refresh_period: Option<Duration>,
}

impl WasmAppRunner {
    pub fn new(
        wasm_bin_path: impl AsRef<Path>,
        refresh_period: Option<Duration>,
        app_name: impl Into<String>,
        serial_conn: SyncConnection,
        display_cfg: DisplayConfiguration,
    ) -> anyhow::Result<Self> {
        let wasm_app_bin = extism::Wasm::file(wasm_bin_path);
        let user_data = extism::UserData::new(PersistentData::new(serial_conn, display_cfg));
        let manifest = extism::Manifest::new([wasm_app_bin]);
        let plugin = with_host_functions(extism::PluginBuilder::new(manifest), &user_data)
            .with_wasi(true)
            .build()?;

        Ok(WasmAppRunner {
            plugin,
            name: app_name.into(),
            refresh_period,
        })
    }

    pub fn from_manifest(
        app_path: impl AsRef<Path>,
        serial_conn: SyncConnection,
        display_cfg: DisplayConfiguration,
    ) -> anyhow::Result<Self> {
        let app_manifest = AppManifest::open(app_path)?;
        Self::new(
            app_manifest.app_bin_path,
            app_manifest.refresh_period,
            app_manifest.app_name,
            serial_conn,
            display_cfg,
        )
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn refresh_period(&self) -> Option<Duration> {
        self.refresh_period
    }

    pub fn setup_app(&mut self) -> anyhow::Result<()> {
        self.plugin.call::<_, ()>("setup", ())
    }

    pub fn run_app_once(&mut self) -> anyhow::Result<()> {
        self.plugin.call::<_, ()>("run", ())
    }
}
