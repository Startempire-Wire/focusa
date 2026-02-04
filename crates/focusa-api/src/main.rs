//! Focusa Daemon — long-lived process hosting cognitive state.
//!
//! Source: docs/G1-12-api.md
//!
//! Default bind: 127.0.0.1:8787
//! No auth in MVP (localhost only).

mod routes;
mod server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "focusa=info".into()),
        )
        .init();

    tracing::info!("Focusa daemon starting");
    server::run().await
}
