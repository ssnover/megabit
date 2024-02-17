use clap::Parser;
use megabit_runner::{
    display::{DisplayConfiguration, PixelRepresentation},
    serial,
    wasm_env::{self, WasmAppRunner},
};
use megabit_serial_protocol::SerialMessage;
use std::path::PathBuf;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone, Debug, Parser)]
pub struct Args {
    /// Path to the tty serial device for the display coprocessor
    #[arg(long)]
    device: PathBuf,
    /// Directory containing megabit app bundles in subdirectories
    #[arg(long)]
    data_dir: Option<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "megabit_runner=debug,megabit_runner::wasm_env::host_functions::host=info".into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    let (serial_conn, serial_task) = serial::start_serial_task(args.device);
    let serial_conn = serial::SyncSerialConnection::new(serial_conn, rt.handle().clone());

    let _serial_task_handle = rt.spawn(Box::into_pin(serial_task));

    let display_info = serial_conn.get_display_info()?;
    let display_info = DisplayConfiguration {
        width: display_info.width as usize,
        height: display_info.height as usize,
        is_rgb: matches!(
            display_info.pixel_representation,
            PixelRepresentation::RGB555
        ),
    };
    tracing::info!("Retrieved info about the display: {display_info:?}");

    let data_dir = args.data_dir.unwrap_or(PathBuf::from(
        [std::env::var("HOME").unwrap().as_str(), "/.megabit"].concat(),
    ));
    if !data_dir.is_dir() {
        tracing::error!("Received invalid path {} to data dir", data_dir.display());
        return Ok(());
    }

    let mut wasm_apps = std::fs::read_dir(data_dir)?
        .into_iter()
        .map(|entry| {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                if let Ok(app) = wasm_env::WasmAppRunner::from_manifest(
                    &path,
                    serial_conn.clone(),
                    display_info.clone(),
                ) {
                    tracing::info!("Found app {} at path: {}", app.name(), path.display());
                    Ok(Some(app))
                } else {
                    tracing::warn!("Unable to build app from bundle at {}", path.display());
                    Ok(None)
                }
            } else {
                Ok(None)
            }
        })
        .filter_map(|app: anyhow::Result<Option<WasmAppRunner>>| match app {
            Ok(app) => app,
            Err(_) => None,
        })
        .collect::<Vec<_>>();
    tracing::info!("Found {} Megabit apps", wasm_apps.len());

    let button_press_matcher =
        Box::new(|msg: &SerialMessage| matches!(msg, &SerialMessage::ReportButtonPress));

    loop {
        for wasm_app in wasm_apps.iter_mut() {
            tracing::info!("Running app: {}", wasm_app.name());
            wasm_app.setup_app()?;

            if let Some(refresh_period) = wasm_app.refresh_period() {
                loop {
                    let start_time = std::time::Instant::now();
                    tracing::debug!("Running");
                    match wasm_app.run_app_once() {
                        Ok(()) => {
                            if start_time.elapsed() < refresh_period {
                                std::thread::sleep(refresh_period - start_time.elapsed())
                            }
                        }
                        Err(err) => {
                            tracing::error!(
                                "Running Wasm app {} failed: {err}, exiting",
                                wasm_app.name()
                            );
                            break;
                        }
                    }
                    if serial_conn
                        .check_for_message_since(button_press_matcher.clone(), start_time)
                        .is_some()
                    {
                        tracing::info!("Button pressed, moving to next app");
                        break;
                    }
                }
            } else {
                // Render and then wait for button press
                if let Ok(()) = wasm_app.run_app_once() {
                    tracing::debug!("Waiting for button press");
                    serial_conn.wait_for_message(button_press_matcher.clone(), None);
                }
            }
        }
    }
}
