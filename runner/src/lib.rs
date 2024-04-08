use display::DisplayConfiguration;
use events::{Event, EventListener};
use std::{io, time::Duration};
use transport::SyncConnection;
use wasm_env::{AppManifest, WasmAppRunner};

pub mod display;
pub mod events;
pub mod transport;
pub mod wasm_env;

const DEFAULT_RUN_PERIOD: Duration = Duration::from_secs(1);

pub struct Runner {
    apps: Vec<AppManifest>,
    current_runner: (usize, WasmAppRunner),
    serial_conn: SyncConnection,
    display_info: DisplayConfiguration,
    event_listener: EventListener,
}

impl Runner {
    pub fn new(
        apps: Vec<AppManifest>,
        serial_conn: SyncConnection,
        display_info: DisplayConfiguration,
    ) -> io::Result<Self> {
        if !apps.is_empty() {
            let initial_app = Self::load_app(&apps[0], serial_conn.clone(), display_info.clone())?;
            Ok(Self {
                apps,
                current_runner: (0, initial_app),
                serial_conn,
                display_info,
                event_listener: EventListener::new(),
            })
        } else {
            todo!("Needs at least one app, write a default app for the future");
        }
    }

    fn load_next_app(&mut self) -> io::Result<()> {
        let next_idx = (self.current_runner.0 + 1) % self.apps.len();
        let app = Self::load_app(
            &self.apps[next_idx],
            self.serial_conn.clone(),
            self.display_info.clone(),
        )?;
        self.current_runner = (next_idx, app);
        Ok(())
    }

    fn load_app(
        manifest: &AppManifest,
        serial_conn: SyncConnection,
        display_info: DisplayConfiguration,
    ) -> io::Result<WasmAppRunner> {
        WasmAppRunner::from_manifest(&manifest.app_bin_path, serial_conn, display_info).map_err(
            |err| {
                tracing::error!(
                    "Failed to load WebAssembly binary for app {}: {:?}",
                    manifest.app_name,
                    err
                );
                io::ErrorKind::InvalidData.into()
            },
        )
    }

    pub fn run(&mut self) {
        loop {
            self.run_app(false);
            while let Some(event) = self.event_listener.next() {
                match event {
                    Event::NextAppRequest
                }
            }
        }
    }

    fn run_app(&mut self, resume: bool) {
        let current_app = &self.apps[self.current_runner.0];
        if !resume {
            tracing::info!(
                "Starting app {} [{}]",
                current_app.app_name,
                current_app.md5sum
            );
            self.current_runner.1.setup_app().unwrap();
        }

        let refresh_period = current_app.refresh_period.unwrap_or(DEFAULT_RUN_PERIOD);
        loop {
            let start_time = std::time::Instant::now();
            tracing::debug!(
                "Running app {} [{}]",
                current_app.app_name,
                current_app.md5sum
            );
            if let Err(err) = self.current_runner.1.run_app_once() {
                tracing::error!("Running Wasm app failed: {err}, exiting");
                break;
            }
            if start_time.elapsed() < refresh_period {
                std::thread::sleep(refresh_period - start_time.elapsed())
            }
            if self.event_listener.has_pending_events() {
                break;
            }
        }
    }
}
