use std::sync::Arc;

use anthropic::messages::CreateMessageRequestWithStream;
use anthropic::messages::Requester;
use anyhow::Result;
use reqwest::RequestBuilder;

#[derive(Clone)]
pub struct Client(Arc<dyn Requester>);

impl Client {
    pub fn new(requester: Arc<dyn Requester>) -> Self {
        Self(requester)
    }
}

#[async_trait::async_trait]
impl Requester for Client {
    fn base_url(&self) -> String {
        self.0.base_url()
    }

    fn endpoint_url(&self, body: &CreateMessageRequestWithStream) -> String {
        self.0.endpoint_url(body)
    }

    async fn request_builder(
        &self,
        url: String,
        body: CreateMessageRequestWithStream,
    ) -> Result<RequestBuilder> {
        self.0.request_builder(url, body).await
    }
}
