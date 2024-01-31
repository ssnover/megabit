use clap::Parser;
use std::{path::PathBuf, time::Duration};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use megabit_runner::serial;

#[derive(Clone, Debug, Parser)]
pub struct Args {
    #[arg(short, long)]
    device: PathBuf,
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
    let (serial_conn, serial_task) = serial::start_serial_task(args.device, tx);
    let serial_conn = serial::SyncSerialConnection::new(serial_conn, rt.handle().clone());

    let serial_task_handle = rt.spawn(Box::into_pin(serial_task));

    for _ in 0..100 {
        serial_conn.set_led_state(true)?;
        std::thread::sleep(Duration::from_secs(1));
        serial_conn.set_led_state(false)?;
        std::thread::sleep(Duration::from_secs(1));
    }

    Ok(())
}
