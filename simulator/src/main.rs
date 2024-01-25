#[cfg(feature = "backend")]
mod backend;

#[cfg(feature = "frontend")]
mod frontend;

mod messages;

#[cfg(feature = "backend")]
#[tokio::main]
async fn main() {
    use backend::*;
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "megabit_coproc_simulator=debug,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let (from_ws_tx, from_ws_rx) = async_channel::unbounded();
    let (to_ws_tx, to_ws_rx) = async_channel::unbounded();
    let (from_serial_tx, from_serial_rx) = async_channel::unbounded();
    let (to_serial_tx, to_serial_rx) = async_channel::unbounded();

    tokio::select! {
        _ = web_server::serve(8000, to_ws_rx, from_ws_tx) => {
            tracing::info!("HTTP server exited");
        },
        _ = simulator::run(from_ws_rx, from_serial_rx, to_ws_tx, to_serial_tx) => {
            tracing::info!("Simulator exited");
        }
        _ = serial::run("/tmp/megabit-sim", from_serial_tx, to_serial_rx) => {
            tracing::info!("Virtual TTY exited");
        }
    };
}

#[cfg(feature = "frontend")]
fn main() {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Info));
    yew::Renderer::<frontend::App>::new().render();
}
