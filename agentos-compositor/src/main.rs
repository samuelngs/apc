mod bitmap;
mod cursor;
mod mcp;
mod state;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    tracing::info!("AgentOS compositor starting");

    #[cfg(target_os = "linux")]
    {
        state::run()?;
    }

    #[cfg(not(target_os = "linux"))]
    {
        tracing::warn!("Compositor only runs inside Linux guest VM");
    }

    Ok(())
}
