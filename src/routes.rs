use std::{convert::Infallible, str::FromStr};

use anthropic::{
    messages::{
        CreateMessageRequestWithStream, IncomingCreateMessageRequest, Messages, MessagesStream,
    },
    Model as AnthropicModel,
};
use anthropic_vertexai::Model as VertexAiModel;
use anyhow::Result;
use axum::{
    extract::State,
    http::StatusCode,
    response::{sse::Event, IntoResponse, Response, Sse},
    Json,
};
use futures::StreamExt;
use serde_json::json;

use crate::{client::Client, provider::Provider};

fn get_model(model: String, provider: Provider) -> Result<String> {
    let model: AnthropicModel = AnthropicModel::from_str(&model)?;

    match provider {
        Provider::Anthropic { .. } => Ok(model.to_string()),
        Provider::VertexAi { .. } => Ok(match model {
            AnthropicModel::ClaudeThreeDotFiveSonnet => VertexAiModel::ClaudeThreeDotFiveSonnet,
            AnthropicModel::ClaudeThreeSonnet => VertexAiModel::ClaudeThreeSonnet,
            AnthropicModel::ClaudeThreeOpus => VertexAiModel::ClaudeThreeOpus,
            AnthropicModel::ClaudeThreeHaiku => VertexAiModel::ClaudeThreeHaiku,
        }
        .to_string()),
    }
}

pub async fn messages(
    State(client): State<Client>,
    State(provider): State<Provider>,
    Json(request): Json<IncomingCreateMessageRequest>,
) -> Response {
    let IncomingCreateMessageRequest {
        stream,
        mut create_message_request,
    } = request;

    create_message_request.model =
        match get_model(create_message_request.model.to_owned(), provider.to_owned()) {
            Ok(model) => model,
            Err(e) => {
                return Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(axum::body::Body::from(
                        json!({ "error": e.to_string() }).to_string(),
                    ))
                    .unwrap()
            }
        };

    if stream.is_some_and(|f| f) {
        let stream = client
            .messages_stream(create_message_request)
            .await
            .unwrap();

        let stream = stream.map(|item| {
            Ok::<Event, Infallible>(
                Event::default().data(&serde_json::to_string(&item.unwrap()).unwrap()),
            )
        });

        Sse::new(stream)
            .keep_alive(axum::response::sse::KeepAlive::new().text("keep-alive-text"))
            .into_response()
    } else {
        Json(client.messages(create_message_request).await.unwrap()).into_response()
    }
}
