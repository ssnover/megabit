use async_channel::{Receiver, Sender};
use axum::{
    extract::{
        ws::{CloseFrame, Message, WebSocket},
        ConnectInfo, State, WebSocketUpgrade,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use std::{net::SocketAddr, ops::ControlFlow};
use tower_http::{
    services::ServeDir,
    trace::{DefaultMakeSpan, TraceLayer},
};

#[derive(Clone)]
struct AppState {
    to_ws_handler: Receiver<String>,
    from_ws_handler: Sender<String>,
}

pub async fn serve(port: u16, to_ws_rx: Receiver<String>, from_ws_tx: Sender<String>) {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    let dist_path: String = std::env::var("SIM_DIST_DIR").unwrap_or("./simulator/dist".to_string());
    let state = AppState {
        to_ws_handler: to_ws_rx,
        from_ws_handler: from_ws_tx,
    };
    let app = Router::new()
        .route("/ws", get(ws_handler))
        .fallback_service(ServeDir::new(dist_path).append_index_html_on_directories(true))
        .with_state(state)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        );
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
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
    to_ws_rx: Receiver<String>,
    from_ws_tx: Sender<String>,
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
    from_ws_tx: Sender<String>,
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
    to_ws_rx: Receiver<String>,
) {
    while let Ok(msg_str) = to_ws_rx.recv().await {
        if let Err(err) = sender.send(Message::Text(msg_str)).await {
            tracing::error!("Unable to send message to web client: {err}");
            break;
        }
    }
}

async fn process_message(
    msg: Message,
    from_ws_tx: &Sender<String>,
    peer: SocketAddr,
) -> ControlFlow<(), ()> {
    match msg {
        Message::Text(t) => {
            tracing::info!(">>> {peer} sent str: {t}");
            if from_ws_tx.send(t).await.is_err() {
                tracing::warn!("Receiver hung up, exiting");
                return ControlFlow::Break(());
            }
        }
        Message::Binary(data) => {
            tracing::info!(">>> {peer} sent {} bytes: {data:?}", data.len());
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
