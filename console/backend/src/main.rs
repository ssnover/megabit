use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod web_server;

const DEFAULT_SERVER_PORT: u16 = 8002;

#[derive(Parser)]
struct Args {
    port: Option<u16>,
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

    web_server::serve(port).await?;

    Ok(())
}
