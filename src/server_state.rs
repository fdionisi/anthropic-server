use axum::extract::FromRef;

use crate::client::Client;

#[derive(Clone)]
pub struct ServerState {
    pub anthropic: Client,
    pub token: String,
}

impl ServerState {
    pub fn new(anthropic: Client, token: String) -> Self {
        Self {
            anthropic,
            token: token.to_string(),
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
