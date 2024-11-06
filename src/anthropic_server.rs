mod client;
pub mod provider;
mod routes;
mod server_state;
pub mod usage_reporter;

use std::sync::Arc;

use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use axum_auth_api_key::ApiKey;
use tower_http::trace::TraceLayer;

use crate::{
    client::Client,
    provider::Provider,
    routes::{healthz, messages},
    server_state::ServerState,
    usage_reporter::UsageReporter,
};

pub struct AnthropicServer {
    client: Client,
    provider: Provider,
    usage_reporter: Arc<dyn UsageReporter>,
    api_key: ApiKey,
}

impl AnthropicServer {
    pub fn builder() -> AnthropicServerBuilder {
        AnthropicServerBuilder::default()
    }

    pub async fn serve<A: tokio::net::ToSocketAddrs>(self, addr: A) -> anyhow::Result<()> {
        let server_state = ServerState::new(self.client, self.provider, self.usage_reporter);

        let app = Router::new()
            .route("/healthz", get(healthz))
            .nest(
                "/v1",
                Router::new()
                    .route("/messages", post(messages))
                    .route_layer(middleware::from_fn_with_state(
                        self.api_key,
                        axum_auth_api_key::auth_middleware,
                    ))
                    .with_state(server_state),
            )
            .layer(TraceLayer::new_for_http());

        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;
        Ok(())
    }
}

#[derive(Default)]
pub struct AnthropicServerBuilder {
    provider: Option<Provider>,
    api_key: Option<ApiKey>,
    usage_reporter: Option<Arc<dyn UsageReporter>>,
}

impl AnthropicServerBuilder {
    pub fn with_provider(mut self, provider: Provider) -> Self {
        self.provider = Some(provider);
        self
    }

    pub fn with_api_key<A>(mut self, api_key: A) -> Self
    where
        A: Into<ApiKey>,
    {
        self.api_key = Some(api_key.into());
        self
    }

    pub fn with_usage_reporter(mut self, usage_reporter: Arc<dyn UsageReporter>) -> Self {
        self.usage_reporter = Some(usage_reporter);
        self
    }

    pub async fn build(self) -> anyhow::Result<AnthropicServer> {
        let provider = self
            .provider
            .ok_or_else(|| anyhow::anyhow!("Provider not set"))?;
        let api_key = self
            .api_key
            .ok_or_else(|| anyhow::anyhow!("API key not set"))?;
        let usage_reporter = self
            .usage_reporter
            .unwrap_or_else(|| Arc::new(crate::usage_reporter::NoopUsageReporter));

        let http_client = Arc::new(http_client_reqwest::HttpClientReqwest::default());

        let client = Client::new(match provider.clone() {
            Provider::Anthropic { api_key } => Arc::new(
                anthropic::Anthropic::builder()
                    .with_http_client(http_client)
                    .with_api_key(api_key)
                    .build()?,
            ),
            Provider::Bedrock => {
                use aws_config::{defaults, BehaviorVersion};
                let config = defaults(BehaviorVersion::latest()).load().await;
                Arc::new(anthropic_bedrock::AnthropicBedrock::new(&config))
            }
            Provider::VertexAi { project, region } => Arc::new(
                anthropic_vertexai::AnthropicVertexAi::builder()
                    .with_http_client(http_client)
                    .with_project(project)
                    .with_region(region)
                    .build()
                    .await?,
            ),
        });

        Ok(AnthropicServer {
            client,
            provider,
            usage_reporter,
            api_key,
        })
    }
}
