use clap::Parser;
use std::{path::PathBuf, time::Duration};
use tokio_serial::SerialPortBuilderExt;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod serial;

#[derive(Clone, Debug, Parser)]
pub struct Args {
    #[arg(short, long)]
    device: PathBuf,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "megabit_runner=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let (tx, _rx) = async_channel::unbounded();

    let serial_conn = serial::SerialConnection::new(args.device, tx);

    let colors = [(0xff, 0x00, 0x00), (0x00, 0xff, 0x00), (0x00, 0x00, 0xff)];

    for i in 0..100 {
        serial_conn.set_led_state(true).await?;
        serial_conn.set_rgb_state(colors[i % colors.len()]).await?;
        tokio::time::sleep(Duration::from_secs(1)).await;
        serial_conn.set_led_state(false).await?;
        serial_conn.set_rgb_state((0x00, 0x00, 0x00)).await?;
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    Ok(())
}
