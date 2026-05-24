#[cfg(target_os = "macos")]
use std::io::{BufRead, BufReader, Write};
#[cfg(target_os = "macos")]
use std::os::unix::net::UnixStream;

#[cfg(target_os = "macos")]
use agentos_protocol::{JsonRpcRequest, JsonRpcResponse};

#[cfg(target_os = "macos")]
pub struct McpClient {
    writer: UnixStream,
    reader: BufReader<UnixStream>,
}

#[cfg(target_os = "macos")]
impl McpClient {
    pub fn connect(socket_path: &str) -> anyhow::Result<Self> {
        let stream = UnixStream::connect(socket_path)?;
        let reader = BufReader::new(stream.try_clone()?);
        tracing::info!(socket_path, "MCP client connected to guest");
        Ok(Self {
            writer: stream,
            reader,
        })
    }

    pub fn call(
        &mut self,
        method: &str,
        params: Option<serde_json::Value>,
    ) -> anyhow::Result<JsonRpcResponse> {
        static ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        let request = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: serde_json::Value::Number(
                ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed).into(),
            ),
            method: method.into(),
            params,
        };

        let mut data = serde_json::to_vec(&request)?;
        data.push(b'\n');
        self.writer.write_all(&data)?;
        self.writer.flush()?;

        let mut line = String::new();
        self.reader.read_line(&mut line)?;
        let response: JsonRpcResponse = serde_json::from_str(&line)?;
        Ok(response)
    }
}

#[cfg(target_os = "macos")]
pub fn run_mcp_test(socket_path: &str) {
    let path = socket_path.to_string();
    std::thread::Builder::new()
        .name("mcp-test".into())
        .spawn(move || {
            for attempt in 0..12 {
                std::thread::sleep(std::time::Duration::from_secs(5));
                tracing::info!(attempt = attempt + 1, "MCP test: attempting connection");
                match McpClient::connect(&path) {
                    Ok(mut client) => {
                        run_test_suite(&mut client);
                        return;
                    }
                    Err(e) => tracing::info!(attempt = attempt + 1, "MCP connect retry: {e}"),
                }
            }
            tracing::error!("MCP test: failed to connect after 60s");
        })
        .expect("failed to spawn MCP test thread");
}

#[cfg(target_os = "macos")]
fn run_test_suite(client: &mut McpClient) {
    tracing::info!("=== MCP TEST SUITE ===");

    let tests: Vec<(&str, serde_json::Value)> = vec![
        (
            "ShellExec (hostname)",
            serde_json::json!({"tool": "shell_exec", "params": {"cmd": "hostname"}}),
        ),
        (
            "ShellExec (uname -a)",
            serde_json::json!({"tool": "shell_exec", "params": {"cmd": "uname -a"}}),
        ),
        (
            "WindowList",
            serde_json::json!({"tool": "window_list"}),
        ),
        (
            "MouseMove (500, 300)",
            serde_json::json!({"tool": "mouse_move", "params": {"x": 500, "y": 300}}),
        ),
        (
            "ScreenCapture",
            serde_json::json!({"tool": "screen_capture", "params": {}}),
        ),
        (
            "WindowOpen (foot)",
            serde_json::json!({"tool": "window_open", "params": {"cmd": "foot"}}),
        ),
    ];

    for (name, params) in &tests {
        tracing::info!("Test: {name}");
        match client.call("tool_call", Some(params.clone())) {
            Ok(resp) => {
                if let Some(result) = &resp.result {
                    tracing::info!("  OK: {}", serde_json::to_string_pretty(result).unwrap_or_default());
                } else if let Some(error) = &resp.error {
                    tracing::error!("  ERROR: {} ({})", error.message, error.code);
                }
            }
            Err(e) => tracing::error!("  TRANSPORT ERROR: {e}"),
        }
    }

    std::thread::sleep(std::time::Duration::from_secs(2));

    let post_tests: Vec<(&str, serde_json::Value)> = vec![
        (
            "WindowList (after open)",
            serde_json::json!({"tool": "window_list"}),
        ),
        (
            "WindowMove (id=0, 100,50)",
            serde_json::json!({"tool": "window_move", "params": {"id": 0, "x": 100, "y": 50}}),
        ),
        (
            "WindowResize (id=0, 800x600)",
            serde_json::json!({"tool": "window_resize", "params": {"id": 0, "width": 800, "height": 600}}),
        ),
        (
            "WindowFocus (id=0)",
            serde_json::json!({"tool": "window_focus", "params": {"id": 0}}),
        ),
        (
            "MouseClick (left)",
            serde_json::json!({"tool": "mouse_click", "params": {"button": "left"}}),
        ),
        (
            "FileWrite (/tmp/mcp_test.txt)",
            serde_json::json!({"tool": "file_write", "params": {"path": "/tmp/mcp_test.txt", "data": [104,101,108,108,111]}}),
        ),
        (
            "FileRead (/tmp/mcp_test.txt)",
            serde_json::json!({"tool": "file_read", "params": {"path": "/tmp/mcp_test.txt"}}),
        ),
    ];

    for (name, params) in &post_tests {
        tracing::info!("Test: {name}");
        match client.call("tool_call", Some(params.clone())) {
            Ok(resp) => {
                if let Some(result) = &resp.result {
                    tracing::info!("  OK: {}", serde_json::to_string_pretty(result).unwrap_or_default());
                } else if let Some(error) = &resp.error {
                    tracing::error!("  ERROR: {} ({})", error.message, error.code);
                }
            }
            Err(e) => tracing::error!("  TRANSPORT ERROR: {e}"),
        }
    }

    tracing::info!("=== MCP TEST COMPLETE ===");
}
