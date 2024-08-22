use std::sync::Arc;

use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use tokio::net::ToSocketAddrs;
use tower_http::trace::TraceLayer;

use crate::{
    auth::auth_middleware,
    client::Client,
    provider::Provider,
    routes::{healthz, messages},
    server_state::ServerState,
};

/// Anthropic proxy server.
///
/// The server allows to communicate with various Anthropic providers, maintaining a single API for all of them.
#[derive(clap::Parser)]
pub struct Cli {
    /// The token used for authenticating all incoming requests
    #[clap(long, env = "AUTH_TOKEN")]
    auth_token: String,
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

impl Cli {
    pub fn address(&self) -> impl ToSocketAddrs {
        (self.host.to_owned(), self.port)
    }
    pub async fn http(&self) -> anyhow::Result<Router> {
        let anthropic: Client = Client::new(match self.provider.to_owned() {
            Provider::Anthropic { api_key } => {
                use anthropic::Anthropic;
                Arc::new(Anthropic::builder().api_key(api_key).build()?)
            }
            Provider::Bedrock => {
                use anthropic_bedrock::AnthropicBedrock;
                use aws_config::{defaults, BehaviorVersion};
                let config = defaults(BehaviorVersion::latest()).load().await;

                Arc::new(AnthropicBedrock::new(&config))
            }
            Provider::VertexAi { project, region } => {
                use anthropic_vertexai::AnthropicVertexAi;
                Arc::new(
                    AnthropicVertexAi::builder()
                        .project(project)
                        .region(region)
                        .build()
                        .await?,
                )
            }
        });

        let server_state =
            ServerState::new(anthropic, self.auth_token.clone(), self.provider.clone());

        Ok(Router::new()
            .route("/healthz", get(healthz))
            .nest(
                "/v1",
                Router::new()
                    .route("/messages", post(messages))
                    .route_layer(middleware::from_fn_with_state(
                        server_state.clone(),
                        auth_middleware,
                    ))
                    .with_state(server_state),
            )
            .layer(TraceLayer::new_for_http()))
    }
}
