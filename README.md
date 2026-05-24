# AgentOS

Lightweight VM for AI agents

AgentOS runs a full Linux desktop inside a [libkrun](https://github.com/containers/libkrun) microVM with GPU acceleration, then exposes it to agents via [MCP](https://modelcontextprotocol.io) tools — screen capture, mouse, keyboard, window management, shell, and file I/O.

## Architecture

```
┌─────────────────────────────────────────────┐
│  Host (macOS)                               │
│  agentos-host        ← native macOS window  │
│    ├─ libkrun VM     ← Apple Hypervisor.fw  │
│    ├─ ANGLE          ← OpenGL ES → Metal    │
│    ├─ IOSurface      ← zero-copy display    │
│    └─ MCP client     ← vsock ──┐            │
├─────────────────────────────────┤            │
│  Guest (Linux)                  │            │
│  agentos-compositor  ← Smithay/Wayland      │
│    ├─ DRM/GBM        ← virtio-gpu 3D       │
│    ├─ libinput       ← virtio-input         │
│    └─ MCP server     ← vsock ──┘            │
└─────────────────────────────────────────────┘
```

## Crates

| Crate | Description |
|---|---|
| `agentos-host` | macOS host — VM lifecycle, display, input forwarding |
| `agentos-compositor` | Linux guest — Wayland compositor with DRM backend |
| `agentos-protocol` | Shared MCP tool definitions and JSON-RPC types |

## MCP Tools

| Tool | Description |
|---|---|
| `screen_capture` | Capture the guest display (full or region) |
| `mouse_move` | Move cursor to absolute position |
| `mouse_click` | Click a mouse button |
| `keyboard_type` | Type a string |
| `keyboard_key` | Press a key with optional modifiers |
| `window_list` | List open windows |
| `window_focus` | Focus a window by ID |
| `window_resize` | Resize a window |
| `window_move` | Move a window |
| `window_open` | Launch a program |
| `window_close` | Close a window |
| `shell_exec` | Run a shell command |
| `file_read` | Read a file |
| `file_write` | Write a file |

## Prerequisites

- macOS (Apple Silicon)
- Rust toolchain
- Docker (for guest image build)
- Ninja, Meson, Python 3 (for native dependencies)

## Build

```bash
# Build native dependencies (ANGLE, libepoxy, virglrenderer, libkrunfw, libkrun)
./deps/build-deps.sh

# Build guest disk image (Alpine Linux + compositor)
./guest/build.sh

# Build and run
cargo build -p agentos-host
codesign --entitlements agentos-host/entitlements.plist --force -s - target/debug/agentos-host

target/debug/agentos-host \
  --kernel guest/out/aarch64/vmlinuz \
  --initrd guest/out/aarch64/initramfs \
  --disk guest/out/aarch64/disk.img
```

## License

[MIT](LICENSE)
