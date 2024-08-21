use std::sync::Arc;

use axum::{middleware, routing::post, Router};
use tokio::net::ToSocketAddrs;
use tower_http::trace::TraceLayer;

use crate::{auth::auth_middleware, client::Client, routes::messages, server_state::ServerState};

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

#[derive(Clone, clap::Subcommand)]
enum Provider {
    /// Use the Anthropic API
    Antrhopic {
        #[clap(long, env = "ANTHROPIC_API_KEY")]
        api_key: String,
    },
    /// Use the Google Vertex AI API
    VertexAi {
        #[clap(long, env = "VERTEXAI_PROJECT")]
        project: String,
        #[clap(long, env = "VERTEXAI_REGION")]
        region: String,
    },
}

impl Cli {
    pub fn address(&self) -> impl ToSocketAddrs {
        (self.host.to_owned(), self.port)
    }
    pub async fn http(&self) -> anyhow::Result<Router> {
        let anthropic: Client = Client::new(match self.provider.to_owned() {
            Provider::Antrhopic { api_key } => {
                use anthropic::Anthropic;
                Arc::new(Anthropic::builder().api_key(api_key).build()?)
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

        let server_state = ServerState::new(anthropic, self.auth_token.clone());

        Ok(Router::new()
            .route("/v1/messages", post(messages))
            .layer(middleware::from_fn_with_state(
                server_state.clone(),
                auth_middleware,
            ))
            .layer(TraceLayer::new_for_http())
            .with_state(server_state))
    }
}
