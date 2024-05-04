use clap::Parser;
use megabit_runner::{
    api_server,
    display::{DisplayConfiguration, PixelRepresentation, ScreenBuffer},
    events::EventListener,
    transport::{self, DeviceTransport},
    wasm_env::{self, AppManifest},
    Runner,
};
use std::path::PathBuf;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone, Debug, Parser)]
pub struct Args {
    /// Path to the tty serial device for the display coprocessor
    #[arg(long)]
    device: DeviceTransport,
    /// Directory containing megabit app bundles in subdirectories
    #[arg(long)]
    data_dir: Option<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "megabit_runner=debug,megabit_runner::wasm_env::host_functions::host=info,megabit_runner::transport=debug".into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    let (serial_conn, serial_task) = transport::start_transport_task(args.device);
    let sync_serial_conn = transport::SyncConnection::new(serial_conn.clone(), rt.handle().clone());

    let _serial_task_handle = rt.spawn(Box::into_pin(serial_task));

    let display_info = sync_serial_conn.get_display_info()?;
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

    let wasm_apps = std::fs::read_dir(data_dir)?
        .map(|entry| {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                if let Ok(app) = wasm_env::AppManifest::open(&path) {
                    tracing::info!("Found app {} at path: {}", &app.app_name, path.display());
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
        .collect::<Vec<_>>();
    tracing::info!("Found {} Megabit apps", wasm_apps.len());

    let api_server_handle = api_server::start(8003, rt.handle().clone());
    let event_listener =
        EventListener::new(serial_conn, api_server_handle.clone(), rt.handle().clone());
    let screen_buffer = ScreenBuffer::create(display_info.width, display_info.height);

    let mut runner = Runner::new(
        wasm_apps,
        sync_serial_conn,
        screen_buffer,
        event_listener,
        api_server_handle,
    )?;
    runner.run();

    tracing::info!("Exiting runner");

    Ok(())
}
