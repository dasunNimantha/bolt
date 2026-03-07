use serde::{Deserialize, Serialize};
use std::io::{self, BufRead, Read, Write};
use std::net::TcpStream;

const IPC_PORT: u16 = 9817;

#[derive(Serialize, Deserialize)]
struct IpcResponse {
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

fn read_native_message() -> Option<Vec<u8>> {
    let mut len_bytes = [0u8; 4];
    io::stdin().read_exact(&mut len_bytes).ok()?;
    let len = u32::from_le_bytes(len_bytes) as usize;
    if len == 0 || len > 1024 * 1024 {
        return None;
    }
    let mut buf = vec![0u8; len];
    io::stdin().read_exact(&mut buf).ok()?;
    Some(buf)
}

fn write_native_message(msg: &[u8]) {
    let len = msg.len() as u32;
    let _ = io::stdout().write_all(&len.to_le_bytes());
    let _ = io::stdout().write_all(msg);
    let _ = io::stdout().flush();
}

fn send_error(msg: &str) {
    let resp = IpcResponse {
        status: "error".to_string(),
        id: None,
        message: Some(msg.to_string()),
    };
    let json = serde_json::to_vec(&resp).unwrap_or_default();
    write_native_message(&json);
}

fn main() {
    let msg = match read_native_message() {
        Some(m) => m,
        None => {
            send_error("Failed to read native message");
            return;
        }
    };

    let mut stream = match TcpStream::connect(("127.0.0.1", IPC_PORT)) {
        Ok(s) => s,
        Err(_) => {
            send_error("Bolt is not running");
            return;
        }
    };

    let mut payload = msg;
    payload.push(b'\n');
    if stream.write_all(&payload).is_err() {
        send_error("Failed to send to Bolt");
        return;
    }

    let mut response_line = String::new();
    let mut reader = io::BufReader::new(&mut stream);
    match reader.read_line(&mut response_line) {
        Ok(0) | Err(_) => {
            send_error("No response from Bolt");
        }
        Ok(_) => {
            let response: IpcResponse =
                serde_json::from_str(response_line.trim()).unwrap_or(IpcResponse {
                    status: "error".to_string(),
                    id: None,
                    message: Some("Invalid response from Bolt".to_string()),
                });
            let json = serde_json::to_vec(&response).unwrap_or_default();
            write_native_message(&json);
        }
    }
}
