use axum::extract::FromRef;

use crate::{client::Client, provider::Provider};

#[derive(Clone)]
pub struct ServerState {
    anthropic: Client,
    token: String,
    provider: Provider,
}

impl ServerState {
    pub fn new(anthropic: Client, token: String, provider: Provider) -> Self {
        Self {
            anthropic,
            token,
            provider,
        }
    }
}

impl FromRef<ServerState> for String {
    fn from_ref(app_state: &ServerState) -> String {
        app_state.token.clone()
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
