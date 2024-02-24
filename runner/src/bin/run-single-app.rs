use clap::Parser;
use megabit_runner::{
    display::{DisplayConfiguration, PixelRepresentation},
    serial, wasm_env,
};
use std::{path::PathBuf, time::Duration};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone, Debug, Parser)]
pub struct Args {
    /// Path to the tty serial device for the display coprocessor
    #[arg(long)]
    device: PathBuf,
    /// Path to a wasm binary application
    #[arg(long)]
    app: PathBuf,
    /// Milliseconds between running each step of the app
    #[arg(long)]
    refresh: Option<u64>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "run_single_app=debug,megabit_runner=debug".into()),
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

    let mut wasm_app = wasm_env::WasmAppRunner::new(
        args.app,
        args.refresh.map(Duration::from_millis),
        "Demo",
        serial_conn,
        display_info,
    )?;

    tracing::info!("Running app: {}", wasm_app.name());
    wasm_app.setup_app()?;

    if let Some(refresh_period) = wasm_app.refresh_period() {
        loop {
            let start_time = std::time::Instant::now();
            tracing::debug!("Running");
            match wasm_app.run_app_once() {
                Ok(()) => {
                    let run_time = start_time.elapsed();
                    tracing::warn!("Running app took {} us", run_time.as_micros());
                    if run_time < refresh_period {
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
        }
    } else {
        if let Ok(()) = wasm_app.run_app_once() {}
    }

    Ok(())
}
