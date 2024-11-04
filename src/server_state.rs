use axum::extract::FromRef;

use crate::{client::Client, provider::Provider};

#[derive(Clone)]
pub struct ServerState {
    anthropic: Client,
    provider: Provider,
}

impl ServerState {
    pub fn new(anthropic: Client, provider: Provider) -> Self {
        Self {
            anthropic,
            provider,
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
