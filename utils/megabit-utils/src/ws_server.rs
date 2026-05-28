use async_channel::{Receiver, Sender};
use axum::{
    extract::{
        ws::{CloseFrame, Message, WebSocket},
        ConnectInfo, State, WebSocketUpgrade,
    },
    response::IntoResponse,
    Router,
};
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use std::{io, net::SocketAddr, ops::ControlFlow, path::Path};
use tower_http::{
    services::ServeDir,
    trace::{DefaultMakeSpan, TraceLayer},
};

#[derive(Clone)]
struct AppState {
    to_ws_handler: Receiver<Vec<u8>>,
    from_ws_handler: Sender<Vec<u8>>,
}

pub async fn serve(
    port: u16,
    dist_path: impl AsRef<Path>,
    to_ws_rx: Receiver<Vec<u8>>,
    from_ws_tx: Sender<Vec<u8>>,
) -> io::Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;

    tracing::debug!("Listening on {}", listener.local_addr()?);
    let app = Router::new()
        .route("/ws", axum::routing::get(ws_handler))
        .fallback_service(ServeDir::new(dist_path).append_index_html_on_directories(true))
        .with_state(AppState {
            to_ws_handler: to_ws_rx,
            from_ws_handler: from_ws_tx,
        })
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        );
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    tracing::info!("Client at {addr} connected");
    let to_ws = state.to_ws_handler.clone();
    let from_ws = state.from_ws_handler.clone();
    ws.on_failed_upgrade(|err| tracing::error!("Failed to upgrade ws connection: {err}"))
        .on_upgrade(move |socket| handle_socket(socket, addr, to_ws, from_ws))
}

async fn handle_socket(
    mut socket: WebSocket,
    peer: SocketAddr,
    to_ws_rx: Receiver<Vec<u8>>,
    from_ws_tx: Sender<Vec<u8>>,
) {
    if socket.send(Message::Ping(vec![1, 2, 3])).await.is_ok() {
        tracing::info!("Pinged {peer}...");
    } else {
        tracing::error!("Could not send ping to {peer}!");
        return;
    }

    let (sender, receiver) = socket.split();

    tokio::join!(
        handle_incoming_ws_message(receiver, from_ws_tx, peer),
        handle_outgoing_payloads(sender, to_ws_rx)
    );
}

async fn handle_incoming_ws_message(
    mut receiver: SplitStream<WebSocket>,
    from_ws_tx: Sender<Vec<u8>>,
    peer: SocketAddr,
) {
    while let Some(msg) = receiver.next().await {
        if let Ok(msg) = msg {
            if process_message(msg, &from_ws_tx, peer).await.is_break() {
                return;
            }
        } else {
            tracing::error!("Client {peer} abruptly disconnected");
            return;
        }
    }
}

async fn handle_outgoing_payloads(
    mut sender: SplitSink<WebSocket, Message>,
    to_ws_rx: Receiver<Vec<u8>>,
) {
    while let Ok(msg) = to_ws_rx.recv().await {
        if let Err(err) = sender.send(Message::Binary(msg)).await {
            tracing::error!("Unable to send message to web client: {err}");
            break;
        }
    }
}

async fn process_message(
    msg: Message,
    from_ws_tx: &Sender<Vec<u8>>,
    peer: SocketAddr,
) -> ControlFlow<(), ()> {
    match msg {
        Message::Binary(data) => {
            tracing::debug!(">>> {peer} sent {} bytes: {data:?}", data.len());
            if from_ws_tx.send(data).await.is_err() {
                tracing::warn!("Receiver hung up, exiting");
                return ControlFlow::Break(());
            }
        }
        Message::Text(text) => {
            tracing::debug!(">>> {peer} sent text {text}");
        }
        Message::Close(c) => {
            if let Some(CloseFrame { code, reason }) = c {
                tracing::debug!(">>> {peer} sent close with code {code} and reason `{reason}`");
            } else {
                tracing::debug!(">>> {peer} sent close message without CloseFrame");
            }
            return ControlFlow::Break(());
        }
        Message::Pong(_v) | Message::Ping(_v) => {}
    }

    ControlFlow::Continue(())
}
