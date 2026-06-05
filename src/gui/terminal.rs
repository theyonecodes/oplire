use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
};
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use std::io::{Read, Write};
use futures::{sink::SinkExt, stream::StreamExt};
use tracing::error;

pub async fn handle_ws_upgrade(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_terminal_session)
}

async fn handle_terminal_session(socket: WebSocket) {
    let pty_system = NativePtySystem::default();
    let pair = match pty_system.openpty(PtySize {
        rows: 24, cols: 80, pixel_width: 0, pixel_height: 0,
    }) {
        Ok(p) => p,
        Err(e) => { error!("Failed to open PTY: {}", e); return; }
    };

    let cmd = CommandBuilder::new(if cfg!(windows) { "powershell.exe" } else { "bash" });
    let _child = match pair.slave.spawn_command(cmd) {
        Ok(c) => c,
        Err(e) => { error!("Failed to spawn shell: {}", e); return; }
    };
    drop(pair.slave);

    let mut reader = pair.master.try_clone_reader().unwrap();
    let writer = pair.master.take_writer().unwrap();

    let (mut ws_tx, mut ws_rx) = socket.split();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();

    tokio::task::spawn_blocking(move || {
        let mut buf = [0u8; 1024];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    if tx.send(String::from_utf8_lossy(&buf[..n]).to_string()).is_err() { break; }
                }
                Err(_) => break,
            }
        }
    });

    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_tx.send(Message::Text(msg)).await.is_err() { break; }
        }
    });

    let recv_task = tokio::spawn(async move {
        let mut writer = writer;
        while let Some(Ok(Message::Text(text))) = ws_rx.next().await {
            let _ = writer.write_all(text.as_bytes());
        }
    });

    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }
}
