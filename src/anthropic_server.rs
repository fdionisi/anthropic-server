mod client;
pub mod provider;
mod routes;
mod server_state;
pub mod usage_reporter;

use std::sync::Arc;

use axum::{
    middleware::{self, Next},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use tower_http::trace::TraceLayer;

use crate::{
    client::Client,
    provider::Provider,
    routes::{healthz, messages},
    server_state::ServerState,
    usage_reporter::UsageReporter,
};

use axum::body::Body;
use axum::http::Request;
use axum::http::StatusCode;

pub trait AuthMiddleware: Send + Sync + 'static {
    fn authenticate(&self, req: &Request<Body>) -> Result<(), (StatusCode, String)>;
}

pub struct AnthropicServer {
    client: Client,
    provider: Provider,
    usage_reporter: Arc<dyn UsageReporter>,
    auth_middleware: Arc<dyn AuthMiddleware>,
}

impl AnthropicServer {
    pub fn builder() -> AnthropicServerBuilder {
        AnthropicServerBuilder::default()
    }

    pub async fn serve<A: tokio::net::ToSocketAddrs>(self, addr: A) -> anyhow::Result<()> {
        let server_state = ServerState::new(self.client, self.provider, self.usage_reporter);

        let auth_middleware = self.auth_middleware;

        let app = Router::new()
            .route("/healthz", get(healthz))
            .nest(
                "/v1",
                Router::new()
                    .route("/messages", post(messages))
                    .route_layer(middleware::from_fn(
                        move |req: Request<Body>, next: Next| {
                            let auth_middleware = auth_middleware.clone();
                            async move {
                                match auth_middleware.authenticate(&req) {
                                    Err(err) => return err.into_response(),
                                    _ => {}
                                };

                                next.run(req).await
                            }
                        },
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
    auth_middleware: Option<Arc<dyn AuthMiddleware>>,
    usage_reporter: Option<Arc<dyn UsageReporter>>,
}

impl AnthropicServerBuilder {
    pub fn with_auth_middleware<M: AuthMiddleware + 'static>(mut self, middleware: M) -> Self {
        self.auth_middleware = Some(Arc::new(middleware));
        self
    }

    pub fn with_provider(mut self, provider: Provider) -> Self {
        self.provider = Some(provider);
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
        let auth_middleware = self
            .auth_middleware
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
            auth_middleware,
        })
    }
}
