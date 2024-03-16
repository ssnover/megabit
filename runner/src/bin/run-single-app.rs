use clap::Parser;
use inotify::{EventMask, Inotify, WatchMask};
use megabit_runner::{
    display::{DisplayConfiguration, PixelRepresentation},
    transport::{self, DeviceTransport},
    wasm_env,
};
use std::{path::PathBuf, time::Duration};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone, Debug, Parser)]
pub struct Args {
    /// Path to the tty serial device for the display coprocessor
    #[arg(long)]
    device: DeviceTransport,
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
                .unwrap_or_else(|_| "run_single_app=debug,megabit_runner=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    let (serial_conn, serial_task) = transport::start_transport_task(args.device);
    let serial_conn = transport::SyncSerialConnection::new(serial_conn, rt.handle().clone());

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

    let mut inotify = Inotify::init().unwrap();
    inotify
        .watches()
        .add(
            &args.app,
            WatchMask::MODIFY
                | WatchMask::CREATE
                | WatchMask::DELETE
                | WatchMask::ACCESS
                | WatchMask::ATTRIB,
        )
        .unwrap();

    loop {
        let mut wasm_app = wasm_env::WasmAppRunner::new(
            &args.app,
            args.refresh.map(Duration::from_millis),
            "Demo",
            serial_conn.clone(),
            display_info.clone(),
        )?;
        tracing::info!("Running app: {}", wasm_app.name());
        wasm_app.setup_app()?;

        let refresh_period = wasm_app.refresh_period().unwrap_or(Duration::from_secs(1));

        let res = loop {
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
                    break Err(err);
                }
            }
            if check_for_app_file_change(&mut inotify) {
                tracing::info!("App file changed, reloading");
                break Ok(());
            }
        };
        if res.is_err() {
            break;
        }
    }

    Ok(())
}

fn check_for_app_file_change(inotify: &mut Inotify) -> bool {
    let mut buffer = [0u8; 4096];
    if let Ok(events) = inotify.read_events(&mut buffer) {
        events
            .into_iter()
            .find(|event| {
                tracing::info!("Event mask: {:?}", event.mask);
                event.mask.contains(EventMask::MODIFY)
                    || event.mask.contains(EventMask::CREATE)
                    || event.mask.contains(EventMask::DELETE)
                    || event.mask.contains(EventMask::ATTRIB)
            })
            .is_some()
    } else {
        false
    }
}
