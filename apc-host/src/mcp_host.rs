use apc_protocol::JsonRpcResponse;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

const REBOOT_TOOL: &str = "reboot";
static REBOOT_SCHEDULED: AtomicBool = AtomicBool::new(false);

pub(crate) enum HostInterceptResponse {
    Response(Vec<u8>),
    NoResponse,
}

pub(crate) fn try_handle_host_request(
    message: &serde_json::Value,
) -> anyhow::Result<Option<HostInterceptResponse>> {
    if tool_call_name(message) != Some(REBOOT_TOOL) {
        return Ok(None);
    }

    let schedule_result = schedule_host_restart();
    if message.get("id").is_none() {
        if let Err(e) = schedule_result {
            tracing::error!(%e, "failed to schedule VM reboot notification");
        }
        return Ok(Some(HostInterceptResponse::NoResponse));
    }

    let id = message
        .get("id")
        .cloned()
        .unwrap_or(serde_json::Value::Null);

    let response = match schedule_result {
        Ok(scheduled) => {
            let text = if scheduled {
                "VM reboot scheduled. APC host will restart the microVM."
            } else {
                "VM reboot is already scheduled."
            };
            JsonRpcResponse::success(
                id,
                serde_json::json!({
                    "content": [{ "type": "text", "text": text }]
                }),
            )
        }
        Err(e) => JsonRpcResponse::error(id, -32000, format!("failed to schedule reboot: {e}")),
    };

    Ok(Some(HostInterceptResponse::Response(serde_json::to_vec(
        &response,
    )?)))
}

pub(crate) fn augment_tools_list_response(
    request: &serde_json::Value,
    response: Vec<u8>,
) -> anyhow::Result<Vec<u8>> {
    if !is_tools_list_request(request) {
        return Ok(response);
    }

    let mut value: serde_json::Value = serde_json::from_slice(&response)?;
    let Some(tools) = value
        .get_mut("result")
        .and_then(|result| result.get_mut("tools"))
        .and_then(serde_json::Value::as_array_mut)
    else {
        return Ok(response);
    };

    let already_present = tools
        .iter()
        .any(|tool| tool.get("name").and_then(serde_json::Value::as_str) == Some(REBOOT_TOOL));
    if !already_present {
        tools.push(reboot_schema());
    }

    Ok(serde_json::to_vec(&value)?)
}

fn is_tools_list_request(message: &serde_json::Value) -> bool {
    message.get("method").and_then(serde_json::Value::as_str) == Some("tools/list")
}

fn tool_call_name(message: &serde_json::Value) -> Option<&str> {
    if message.get("method").and_then(serde_json::Value::as_str) != Some("tools/call") {
        return None;
    }
    message
        .get("params")
        .and_then(|params| params.get("name"))
        .and_then(serde_json::Value::as_str)
}

fn reboot_schema() -> serde_json::Value {
    serde_json::json!({
        "name": REBOOT_TOOL,
        "description": "Reboot the APC microVM by restarting the host VM process.",
        "inputSchema": {
            "type": "object",
            "properties": {}
        }
    })
}

fn schedule_host_restart() -> anyhow::Result<bool> {
    if REBOOT_SCHEDULED.swap(true, Ordering::SeqCst) {
        return Ok(false);
    }

    let exe = std::env::current_exe()?;
    let args: Vec<std::ffi::OsString> = std::env::args_os().skip(1).collect();
    spawn_replacement_process(&exe, &args)?;

    std::thread::Builder::new()
        .name("vm-reboot".into())
        .spawn(move || {
            std::thread::sleep(Duration::from_millis(350));
            tracing::info!(
                exe = %exe.display(),
                "exiting APC host for VM reboot"
            );
            unsafe {
                libc::_exit(0);
            }
        })?;

    Ok(true)
}

#[cfg(unix)]
fn spawn_replacement_process(
    exe: &std::path::Path,
    args: &[std::ffi::OsString],
) -> anyhow::Result<()> {
    use std::os::unix::process::CommandExt;
    use std::process::Stdio;

    let max_fd = open_fd_limit();
    let mut command = std::process::Command::new("/bin/sh");
    command
        .arg("-c")
        .arg("sleep 0.8; exec \"$@\"")
        .arg("apc-host-reboot")
        .arg(exe)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    unsafe {
        command.pre_exec(move || {
            if libc::setsid() == -1 {
                return Err(std::io::Error::last_os_error());
            }
            close_inherited_fds(max_fd);
            Ok(())
        });
    }

    command.spawn()?;
    Ok(())
}

#[cfg(unix)]
fn open_fd_limit() -> i32 {
    let max_fd = unsafe { libc::sysconf(libc::_SC_OPEN_MAX) };
    if max_fd > 0 { max_fd as i32 } else { 1024 }
}

#[cfg(unix)]
fn close_inherited_fds(max_fd: i32) {
    for fd in 3..max_fd {
        unsafe {
            libc::close(fd);
        }
    }
}

#[cfg(not(unix))]
fn spawn_replacement_process(
    _exe: &std::path::Path,
    _args: &[std::ffi::OsString],
) -> anyhow::Result<()> {
    anyhow::bail!("reboot requires a Unix host")
}
