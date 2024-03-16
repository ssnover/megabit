use clap::Parser;
use megabit_runner::transport::{self, DeviceTransport};
use megabit_serial_protocol::SerialMessage;
use std::time::Duration;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone, Debug, Parser)]
pub struct Args {
    #[arg(short, long)]
    device: DeviceTransport,
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

    let (serial_conn, serial_task) = transport::start_transport_task(args.device);
    let _serial_task_handle = tokio::spawn(Box::into_pin(serial_task));

    let colors = [(0xff, 0x00, 0x00), (0x00, 0xff, 0x00), (0x00, 0x00, 0xff)];

    serial_conn
        .wait_for_message(
            Box::new(|msg| matches!(msg, &SerialMessage::ReportButtonPress)),
            None,
        )
        .await;

    let display_info = serial_conn.get_display_info().await.unwrap();

    for row in 0..display_info.height as u8 {
        for col in 0..display_info.width as u8 {
            serial_conn.set_single_cell(row, col, true).await?;
            tokio::time::sleep(Duration::from_millis(100)).await;
            serial_conn.set_single_cell(row, col, false).await?;
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
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
