use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::unix::net::UnixStream;
use std::time::{Duration, Instant};

/// Run a line-delimited JSON-RPC stdio proxy.
///
/// Reads lines from stdin, forwards each to the guest VM's MCP server via
/// Unix socket at `socket_path`, reads the response line, and writes it to
/// stdout. Exits when stdin reaches EOF.
pub fn run_stdio_proxy(socket_path: &str) -> anyhow::Result<()> {
    let stream = connect_with_retry(socket_path)?;
    let mut sock_reader = BufReader::new(stream.try_clone()?);
    let mut sock_writer = BufWriter::new(stream);

    let stdin = std::io::stdin();
    let stdin = stdin.lock();
    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();

    for line_result in stdin.lines() {
        let line = match line_result {
            Ok(l) => l,
            Err(e) => {
                tracing::error!("failed to read from stdin: {e}");
                break;
            }
        };

        // Forward request to socket (line-delimited)
        if let Err(e) = sock_writer
            .write_all(line.as_bytes())
            .and_then(|_| sock_writer.write_all(b"\n"))
            .and_then(|_| sock_writer.flush())
        {
            tracing::error!("failed to write to socket: {e}");
            break;
        }

        // Read response from socket
        let mut response = String::new();
        match sock_reader.read_line(&mut response) {
            Ok(0) => {
                tracing::info!("socket closed by guest");
                break;
            }
            Ok(_) => {}
            Err(e) => {
                tracing::error!("failed to read from socket: {e}");
                break;
            }
        }

        // Write response to stdout (line-delimited)
        if let Err(e) = stdout
            .write_all(response.as_bytes())
            .and_then(|_| {
                // Ensure the line ends with a newline
                if !response.ends_with('\n') {
                    stdout.write_all(b"\n")
                } else {
                    Ok(())
                }
            })
            .and_then(|_| stdout.flush())
        {
            tracing::error!("failed to write to stdout: {e}");
            break;
        }
    }

    tracing::info!("stdio proxy shutting down");
    Ok(())
}

/// Connect to the Unix socket, retrying every 2 seconds for up to 60 seconds.
fn connect_with_retry(socket_path: &str) -> anyhow::Result<UnixStream> {
    let deadline = Instant::now() + Duration::from_secs(60);
    let mut attempt: u32 = 0;

    loop {
        attempt += 1;
        tracing::info!(attempt, socket_path, "connecting to MCP socket");

        match UnixStream::connect(socket_path) {
            Ok(stream) => {
                tracing::info!(socket_path, "MCP stdio proxy connected");
                return Ok(stream);
            }
            Err(e) => {
                if Instant::now() >= deadline {
                    return Err(anyhow::anyhow!(
                        "failed to connect to {socket_path} after {attempt} attempts: {e}"
                    ));
                }
                tracing::info!(attempt, "connection failed ({e}), retrying in 2s");
                std::thread::sleep(Duration::from_secs(2));
            }
        }
    }
}
