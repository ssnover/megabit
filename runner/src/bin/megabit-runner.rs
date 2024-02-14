use clap::Parser;
use megabit_runner::{display::DisplayConfiguration, serial, wasm_env};
use megabit_serial_protocol::PixelRepresentation;
use std::path::PathBuf;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone, Debug, Parser)]
pub struct Args {
    #[arg(short, long)]
    device: PathBuf,
    /// Directory containing an app manifest
    #[arg(short, long)]
    app: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "megabit_runner=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    let (tx, rx) = async_channel::unbounded();
    let (serial_conn, serial_task) = serial::start_serial_task(args.device, tx, rx);
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

    let mut wasm_app = wasm_env::WasmAppRunner::new(args.app, serial_conn, display_info)?;
    tracing::info!("Running app: {}", wasm_app.name());
    wasm_app.setup_app()?;

    if let Some(refresh_period) = wasm_app.refresh_period() {
        loop {
            let start_time = std::time::Instant::now();
            match wasm_app.run_app_once() {
                Ok(()) => std::thread::sleep(start_time.elapsed() - refresh_period),
                Err(err) => {
                    tracing::error!(
                        "Running Wasm app {} failed: {err}, exiting",
                        wasm_app.name()
                    );
                    break;
                }
            }
        }
    }

    Ok(())
}
