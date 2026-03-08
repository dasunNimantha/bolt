use crate::model::PendingDownload;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;

pub const IPC_PORT: u16 = 9817;

#[derive(Deserialize)]
struct IpcRequest {
    #[serde(default)]
    url: String,
    filename: Option<String>,
    referrer: Option<String>,
    cookies: Option<String>,
    #[serde(default)]
    ping: bool,
}

#[derive(Serialize)]
struct IpcResponse {
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

pub async fn start_ipc_server(pending: Arc<Mutex<Vec<PendingDownload>>>) {
    let listener = match TcpListener::bind(("127.0.0.1", IPC_PORT)).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("IPC server failed to bind on port {}: {}", IPC_PORT, e);
            return;
        }
    };

    loop {
        let (stream, _) = match listener.accept().await {
            Ok(conn) => conn,
            Err(_) => continue,
        };

        let pending = pending.clone();

        tokio::spawn(async move {
            let _ = handle_connection(stream, pending).await;
        });
    }
}

async fn handle_connection(
    stream: tokio::net::TcpStream,
    pending: Arc<Mutex<Vec<PendingDownload>>>,
) -> anyhow::Result<()> {
    let (reader, mut writer) = stream.into_split();
    let mut lines = BufReader::new(reader).lines();

    while let Ok(Some(line)) = lines.next_line().await {
        let response = process_request(&line, &pending);
        let mut json = serde_json::to_string(&response).unwrap_or_default();
        json.push('\n');
        writer.write_all(json.as_bytes()).await?;
    }
    Ok(())
}

fn process_request(line: &str, pending: &Arc<Mutex<Vec<PendingDownload>>>) -> IpcResponse {
    let req: IpcRequest = match serde_json::from_str(line) {
        Ok(r) => r,
        Err(e) => {
            return IpcResponse {
                status: "error".to_string(),
                id: None,
                message: Some(format!("Invalid JSON: {}", e)),
            };
        }
    };

    if req.ping {
        return IpcResponse {
            status: "ok".to_string(),
            id: None,
            message: Some("pong".to_string()),
        };
    }

    if req.url.is_empty() {
        return IpcResponse {
            status: "error".to_string(),
            id: None,
            message: Some("Missing url field".to_string()),
        };
    }

    let mut headers = HashMap::new();
    if let Some(ref r) = req.referrer {
        if !r.is_empty() {
            headers.insert("Referer".to_string(), r.clone());
        }
    }
    if let Some(ref c) = req.cookies {
        if !c.is_empty() {
            headers.insert("Cookie".to_string(), c.clone());
        }
    }

    pending.lock().unwrap().push(PendingDownload {
        url: req.url,
        filename: req.filename,
        headers,
        resolved: None,
    });

    IpcResponse {
        status: "ok".to_string(),
        id: None,
        message: None,
    }
}
