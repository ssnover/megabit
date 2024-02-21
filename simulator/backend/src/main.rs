mod backend;

#[tokio::main]
async fn main() {
    use backend::*;
    use clap::Parser;
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

    #[derive(Parser)]
    struct Args {
        #[arg(long)]
        is_monocolor: Option<bool>,
    }

    let args = Args::parse();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "megabit_coproc_simulator=info,megabit_serial_protocol=warn,tower_http=info".into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let display_cfg = if !args.is_monocolor.unwrap_or(false) {
        DisplayConfiguration {
            is_rgb: true,
            width: 64,
            height: 32,
        }
    } else {
        DisplayConfiguration {
            is_rgb: false,
            width: 32,
            height: 16,
        }
    };
    tracing::info!("Simulating display with config: {:?}", display_cfg);

    let (from_ws_tx, from_ws_rx) = async_channel::unbounded();
    let (to_ws_tx, to_ws_rx) = async_channel::unbounded();
    let (from_serial_tx, from_serial_rx) = async_channel::unbounded();
    let (to_serial_tx, to_serial_rx) = async_channel::unbounded();

    tokio::select! {
        _ = web_server::serve(8000, to_ws_rx, from_ws_tx) => {
            tracing::info!("HTTP server exited");
        },
        _ = simulator::run(from_ws_rx, from_serial_rx, to_ws_tx, to_serial_tx, display_cfg) => {
            tracing::info!("Simulator exited");
        }
        _ = serial::run(9009, from_serial_tx, to_serial_rx) => {
            tracing::info!("Virtual TTY exited");
        }
    };
}
