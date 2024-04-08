use async_channel::{Receiver, Sender};
use std::io;

pub async fn serve(
    port: u16,
    to_ws_rx: Receiver<Vec<u8>>,
    from_ws_tx: Sender<Vec<u8>>,
) -> io::Result<()> {
    let dist_path = std::env::var("CONSOLE_DIST_DIR").unwrap_or("./console/frontend/dist".into());
    megabit_utils::serve(port, dist_path, to_ws_rx, from_ws_tx).await
}
