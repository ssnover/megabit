use std::{io, net::SocketAddr};

use axum::Router;
use tower_http::{
    services::ServeDir,
    trace::{DefaultMakeSpan, TraceLayer},
};

pub async fn serve(port: u16) -> io::Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;

    tracing::debug!("Listening on {}", listener.local_addr()?);
    let dist_path = std::env::var("CONSOLE_DIST_DIR").unwrap_or("./console/frontend/dist".into());
    let app = Router::new()
        .fallback_service(ServeDir::new(dist_path).append_index_html_on_directories(true))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        );
    axum::serve(listener, app.into_make_service()).await
}
