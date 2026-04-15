use apps::AppManifest;
use display::ScreenBufferHandle;
use events::{Event, EventListener};
use std::{io, time::Duration};
use streams::{api_server::ApiServerHandle, coproc_client::SyncConnection};
use wasm_env::WasmAppRunner;

pub mod apps;
pub mod cmd_queue;
pub mod display;
pub mod events;
pub mod streams;
pub mod wasm_env;

const DEFAULT_RUN_PERIOD: Duration = Duration::from_secs(1);

pub struct Runner {
    app_library: apps::Library,
    is_running: bool,
    runner: WasmAppRunner,
    serial_conn: SyncConnection,
    screen_buffer: ScreenBufferHandle,
    event_listener: EventListener,
    api_server: ApiServerHandle,
}

impl Runner {
    pub fn new(
        app_library: apps::Library,
        serial_conn: SyncConnection,
        screen_buffer: ScreenBufferHandle,
        event_listener: EventListener,
        api_server: ApiServerHandle,
    ) -> io::Result<Self> {
        if let Some(app) = app_library.get_first() {
            let initial_app = Self::load_app(
                &app,
                serial_conn.clone(),
                screen_buffer.clone(),
                api_server.clone(),
            )?;
            Ok(Self {
                app_library,
                is_running: true,
                runner: initial_app,
                serial_conn,
                screen_buffer,
                event_listener,
                api_server,
            })
        } else {
            todo!("Needs at least one app, write a default app for the future");
        }
    }

    fn load_next_app(&mut self) -> io::Result<()> {
        if let Some(app) = self.app_library.get_next(self.runner.id()) {
            self.runner = Self::load_app(
                &app,
                self.serial_conn.clone(),
                self.screen_buffer.clone(),
                self.api_server.clone(),
            )?;
            self.is_running = true;
        }
        Ok(())
    }

    fn load_previous_app(&mut self) -> io::Result<()> {
        if let Some(app) = self.app_library.get_prev(self.runner.id()) {
            self.runner = Self::load_app(
                &app,
                self.serial_conn.clone(),
                self.screen_buffer.clone(),
                self.api_server.clone(),
            )?;
            self.is_running = true;
        }
        Ok(())
    }

    fn load_app(
        manifest: &AppManifest,
        serial_conn: SyncConnection,
        screen_buffer: ScreenBufferHandle,
        api_server: ApiServerHandle,
    ) -> io::Result<WasmAppRunner> {
        WasmAppRunner::from_manifest(
            &manifest.app_bin_path.parent().unwrap(),
            serial_conn,
            screen_buffer,
            api_server,
        )
        .map_err(|err| {
            tracing::error!(
                "Failed to load WebAssembly binary for app {}: {:?}",
                manifest.app_name,
                err
            );
            io::ErrorKind::InvalidData.into()
        })
        .map(|runner| {
            tracing::info!("Loaded WebAssembly binary for app {}", &manifest.app_name);
            runner
        })
    }

    pub fn run(&mut self) {
        loop {
            if self.is_running {
                self.run_app(false);
            }
            while let Some(event) = self.event_listener.next() {
                match event {
                    Event::NextAppRequest => {
                        let current_app = self.runner.id().to_string();

                        while let Err(err) = self.load_next_app() {
                            if &current_app == self.runner.id() {
                                // We've fully cycled around...
                                tracing::error!("Cannot load any apps, exiting");
                                std::process::exit(1);
                            }
                            tracing::error!("Unable to load app: {err:?}, going to the next");
                        }
                    }
                    Event::PreviousAppRequest => {
                        let current_app = self.runner.id().to_string();

                        while let Err(err) = self.load_previous_app() {
                            if &current_app == self.runner.id() {
                                // We've fully cycled around...
                                tracing::error!("Cannot load any apps, exiting");
                                std::process::exit(1);
                            }
                            tracing::error!("Unable to load app: {err:?}, going to the next");
                        }
                    }
                    Event::ReloadAppsRequest => {
                        tracing::error!("Unsupported");
                    }
                    Event::ResumePauseRequest => {
                        match self.is_running {
                            true => {
                                tracing::info!("Pausing execution of the runner");
                            }
                            false => {
                                tracing::info!("Resuming execution of the runner");
                            }
                        }
                        self.is_running = !self.is_running;
                    }
                    Event::Shutdown => {
                        tracing::info!("Received shutdown, stopping runner");
                        return;
                    }
                }
            }
        }
    }

    fn run_app(&mut self, resume: bool) {
        if !resume {
            tracing::info!("Starting app {} [{}]", self.runner.name(), self.runner.id());
            self.runner.setup_app().unwrap();
        }

        let refresh_period = self.runner.refresh_period().unwrap_or(DEFAULT_RUN_PERIOD);
        loop {
            let start_time = std::time::Instant::now();
            tracing::debug!("Running app {} [{}]", self.runner.name(), self.runner.id());
            if let Err(err) = self.runner.run_app_once() {
                tracing::error!("Running Wasm app failed: {err}, exiting");
                break;
            }
            tracing::debug!("Finished running app");
            if start_time.elapsed() < refresh_period {
                std::thread::sleep(refresh_period - start_time.elapsed())
            }
            tracing::debug!("Sleep finished");
            if self.event_listener.has_pending_events() {
                tracing::debug!("Handling events");
                break;
            }
        }
    }
}
