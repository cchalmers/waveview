use std::{net::SocketAddr, path::PathBuf, time::Duration};

use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use clap::Parser;
use futures_util::{SinkExt, StreamExt};
use tokio::{
    io::{AsyncReadExt, AsyncSeekExt},
    sync::broadcast,
};

#[derive(Debug, Parser)]
#[command(about = "Stream a growing VCD file to waveview clients")]
struct Options {
    /// VCD file to snapshot and follow. It may be replaced or truncated by a simulator.
    #[arg(long, short)]
    input: PathBuf,

    #[arg(long, default_value = "127.0.0.1:9123")]
    listen: SocketAddr,

    #[arg(long, default_value_t = 100)]
    poll_ms: u64,
}

#[derive(Clone)]
struct AppState {
    input: PathBuf,
    chunks: broadcast::Sender<StreamEvent>,
}

#[derive(Clone, Debug)]
enum StreamEvent {
    Reset,
    Data(Vec<u8>),
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let options = Options::parse();
    let (chunks, _) = broadcast::channel(256);
    let state = AppState {
        input: options.input.clone(),
        chunks,
    };

    tokio::spawn(follow_file(
        state.clone(),
        Duration::from_millis(options.poll_ms),
    ));

    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/ws", get(websocket))
        .with_state(state);
    let listener = tokio::net::TcpListener::bind(options.listen).await?;
    tracing::info!(address = %options.listen, input = %options.input.display(), "waveserve listening");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn websocket(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| serve_client(socket, state))
}

async fn serve_client(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let hello = serde_json::json!({ "type": "hello", "protocol": "waveview-v1" }).to_string();
    if sender.send(Message::Text(hello.into())).await.is_err() {
        return;
    }
    if let Ok(snapshot) = tokio::fs::read(&state.input).await {
        if sender
            .send(Message::Text(r#"{"type":"reset"}"#.into()))
            .await
            .is_err()
            || sender.send(Message::Binary(snapshot.into())).await.is_err()
        {
            return;
        }
    }

    let mut chunks = state.chunks.subscribe();
    loop {
        tokio::select! {
            message = receiver.next() => match message {
                Some(Ok(Message::Close(_))) | None | Some(Err(_)) => break,
                _ => {}
            },
            event = chunks.recv() => match event {
                Ok(StreamEvent::Reset) => {
                    if sender.send(Message::Text(r#"{"type":"reset"}"#.into())).await.is_err() { break; }
                }
                Ok(StreamEvent::Data(bytes)) => {
                    if sender.send(Message::Binary(bytes.into())).await.is_err() { break; }
                }
                Err(broadcast::error::RecvError::Lagged(_)) => {
                    let _ = sender.send(Message::Text(r#"{"type":"resync"}"#.into())).await;
                    break;
                }
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    }
}

async fn follow_file(state: AppState, poll_interval: Duration) {
    let mut offset = 0;
    loop {
        let metadata = match tokio::fs::metadata(&state.input).await {
            Ok(metadata) => metadata,
            Err(error) => {
                tracing::debug!(%error, path = %state.input.display(), "waiting for VCD");
                tokio::time::sleep(poll_interval).await;
                continue;
            }
        };

        if metadata.len() < offset {
            offset = 0;
            let _ = state.chunks.send(StreamEvent::Reset);
        }

        if metadata.len() > offset {
            match tokio::fs::File::open(&state.input).await {
                Ok(mut file) => {
                    if file.seek(std::io::SeekFrom::Start(offset)).await.is_ok() {
                        let mut bytes = Vec::with_capacity((metadata.len() - offset) as usize);
                        if file.read_to_end(&mut bytes).await.is_ok() && !bytes.is_empty() {
                            offset += bytes.len() as u64;
                            let _ = state.chunks.send(StreamEvent::Data(bytes));
                        }
                    }
                }
                Err(error) => tracing::warn!(%error, "failed to open VCD"),
            }
        }
        tokio::time::sleep(poll_interval).await;
    }
}
