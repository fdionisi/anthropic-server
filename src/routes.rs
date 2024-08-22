use std::{convert::Infallible, str::FromStr};

use anthropic::{
    messages::{CreateMessageResponse, IncomingCreateMessageRequest},
    Model as AnthropicModel,
};
use anthropic_bedrock::Model as BedrockModel;
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

    Ok(match provider {
        Provider::Anthropic { .. } => model.to_string(),
        Provider::Bedrock { .. } => match model {
            AnthropicModel::ClaudeThreeDotFiveSonnet => BedrockModel::ClaudeThreeDotFiveSonnet,
            AnthropicModel::ClaudeThreeSonnet => BedrockModel::ClaudeThreeSonnet,
            AnthropicModel::ClaudeThreeOpus => BedrockModel::ClaudeThreeOpus,
            AnthropicModel::ClaudeThreeHaiku => BedrockModel::ClaudeThreeHaiku,
        }
        .to_string(),
        Provider::VertexAi { .. } => match model {
            AnthropicModel::ClaudeThreeDotFiveSonnet => VertexAiModel::ClaudeThreeDotFiveSonnet,
            AnthropicModel::ClaudeThreeSonnet => VertexAiModel::ClaudeThreeSonnet,
            AnthropicModel::ClaudeThreeOpus => VertexAiModel::ClaudeThreeOpus,
            AnthropicModel::ClaudeThreeHaiku => VertexAiModel::ClaudeThreeHaiku,
        }
        .to_string(),
    })
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
            let item = item.unwrap();
            let item = serde_json::to_value(&item).unwrap();
            Ok::<Event, Infallible>(
                Event::default()
                    .event(item["type"].as_str().unwrap())
                    .data(&serde_json::to_string(&item).unwrap()),
            )
        });

        Sse::new(stream)
            .keep_alive(axum::response::sse::KeepAlive::new())
            .into_response()
    } else {
        match client.messages(create_message_request).await {
            Ok(message_respose) => match message_respose {
                CreateMessageResponse::Message(_) => {
                    (StatusCode::OK, Json(message_respose)).into_response()
                }
                CreateMessageResponse::Error { .. } => {
                    (StatusCode::BAD_REQUEST, Json(message_respose)).into_response()
                }
            },
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": err.to_string() })),
            )
                .into_response(),
        }
    }
}
