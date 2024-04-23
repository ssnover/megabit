use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod runner_client;
mod web_server;

const DEFAULT_SERVER_PORT: u16 = 8002;
const DEFAULT_RUNNER_SERVER_PORT: u16 = 8003;

#[derive(Parser)]
struct Args {
    port: Option<u16>,
    runner_port: Option<u16>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "megabit_console_server=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let port = args.port.unwrap_or(DEFAULT_SERVER_PORT);
    let runner_port = args.runner_port.unwrap_or(DEFAULT_RUNNER_SERVER_PORT);
    let (from_ws_tx, from_ws_rx) = async_channel::unbounded();
    let (to_ws_tx, to_ws_rx) = async_channel::unbounded();

    tokio::select! {
        _ = web_server::serve(port, to_ws_rx, from_ws_tx) => {
            tracing::info!("Web server context exited");
        }
        _ = runner_client::connect(runner_port, from_ws_rx, to_ws_tx) => {
            tracing::info!("Runner client context exited");
        }
    }

    Ok(())
}
