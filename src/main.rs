use anthropic_server::{provider::Provider, AnthropicServer};
use anyhow::Result;
use clap::Parser;
use tokio::net::ToSocketAddrs;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Anthropic proxy server.
///
/// The server allows to communicate with various Anthropic providers, maintaining a single API for all of them.
#[derive(clap::Parser)]
pub struct Cli {
    /// The token used for authenticating all incoming requests
    #[clap(long, env = "AUTH_TOKEN")]
    pub api_key: String,
    /// The host to bind to
    #[clap(long, default_value = "0.0.0.0")]
    host: String,
    /// The port to bind to
    #[clap(long, default_value = "3000")]
    port: u16,
    /// The provider to use for the model
    #[command(subcommand)]
    pub provider: Provider,
}

impl Cli {
    pub fn address(&self) -> impl ToSocketAddrs {
        (self.host.to_owned(), self.port)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "anthropic-server=debug,tower_http=debug,axum::rejection=trace".into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cli = Cli::parse();
    let address = cli.address();

    AnthropicServer::builder()
        .with_provider(cli.provider)
        .with_api_key(cli.api_key)
        .build()
        .await?
        .serve(address)
        .await
}
