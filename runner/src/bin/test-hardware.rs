use clap::Parser;
use megabit_serial_protocol::SerialMessage;
use std::{path::PathBuf, time::Duration};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use megabit_runner::serial;

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

    let (tx, rx) = async_channel::unbounded();

    let (serial_conn, serial_task) = serial::start_serial_task(args.device, tx, rx.clone());
    let _serial_task_handle = tokio::spawn(Box::into_pin(serial_task));

    let colors = [(0xff, 0x00, 0x00), (0x00, 0xff, 0x00), (0x00, 0x00, 0xff)];

    loop {
        let msg = rx.recv().await.unwrap();
        if matches!(msg, SerialMessage::ReportButtonPress) {
            break;
        }
    }

    for row in 0..16 {
        for col in 0..32u8 {
            serial_conn
                .update_row(row, {
                    let mut data = vec![false; 32];
                    data[usize::from(col)] = true;
                    data
                })
                .await?;
            tokio::time::sleep(Duration::from_millis(33)).await;
        }
        serial_conn.update_row(row, vec![false; 32]).await?;
    }

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
