mod auth;
mod cli;
mod client;
mod provider;
mod routes;
mod server_state;

use anyhow::Result;
use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "anthropic-server=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cli = cli::Cli::parse();

    let server = cli.http().await?;
    let listener = tokio::net::TcpListener::bind(cli.address()).await?;

    tracing::debug!("listening on {}", listener.local_addr()?);
    axum::serve(listener, server).await?;

    Ok(())
}
