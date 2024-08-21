use std::{ops::Deref, sync::Arc};

use anthropic::messages::AnthropicSdk;

#[derive(Clone)]
pub struct Client(Arc<dyn AnthropicSdk + Send + Sync>);

impl Client {
    pub fn new(requester: Arc<dyn AnthropicSdk + Send + Sync>) -> Self {
        Self(requester)
    }
}

impl Deref for Client {
    type Target = Arc<dyn AnthropicSdk + Send + Sync>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
