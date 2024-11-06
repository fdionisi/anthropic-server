use anthropic_server::{provider::Provider, AnthropicServer, AuthMiddleware};
use anyhow::Result;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Anthropic Server.
///
/// The server allows to communicate with various Anthropic providers, maintaining a single API for all of them.
#[derive(clap::Parser)]
pub struct Cli {
    /// The token used for authenticating all incoming requests
    #[clap(long, env = "API_KEY")]
    api_key: String,
    /// The host to bind to
    #[clap(long, default_value = "0.0.0.0")]
    host: String,
    /// The port to bind to
    #[clap(long, default_value = "3000")]
    port: u16,
    /// The provider to use for the model
    #[command(subcommand)]
    provider: Provider,
}

struct ApiKeyMiddleware {
    key: String,
}

impl AuthMiddleware for ApiKeyMiddleware {
    fn authenticate(&self, req: &Request<Body>) -> Result<(), (StatusCode, String)> {
        let api_key = req
            .headers()
            .get("x-api-key")
            .ok_or((
                StatusCode::UNAUTHORIZED,
                "Missing x-api-key header".to_string(),
            ))?
            .to_str()
            .map_err(|_| {
                (
                    StatusCode::UNAUTHORIZED,
                    "Invalid x-api-key header".to_string(),
                )
            })?;

        if api_key == self.key {
            Ok(())
        } else {
            Err((StatusCode::UNAUTHORIZED, "Invalid API key".to_string()))
        }
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

    AnthropicServer::builder()
        .with_provider(cli.provider)
        .with_auth_middleware(ApiKeyMiddleware { key: cli.api_key })
        .build()
        .await?
        .serve((cli.host, cli.port))
        .await
}
