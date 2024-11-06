use std::sync::Arc;

use axum::extract::FromRef;

use crate::{client::Client, provider::Provider, usage_reporter::UsageReporter};

#[derive(Clone)]
pub struct ServerState {
    anthropic: Client,
    provider: Provider,
    usage_reporter: Arc<dyn UsageReporter>,
}

impl ServerState {
    pub fn new(
        anthropic: Client,
        provider: Provider,
        usage_reporter: Arc<dyn UsageReporter>,
    ) -> Self {
        Self {
            anthropic,
            provider,
            usage_reporter,
        }
    }
}

impl FromRef<ServerState> for Client {
    fn from_ref(app_state: &ServerState) -> Client {
        app_state.anthropic.clone()
    }
}

impl FromRef<ServerState> for Provider {
    fn from_ref(app_state: &ServerState) -> Provider {
        app_state.provider.clone()
    }
}

impl FromRef<ServerState> for Arc<dyn UsageReporter> {
    fn from_ref(app_state: &ServerState) -> Arc<dyn UsageReporter> {
        app_state.usage_reporter.clone()
    }
}
