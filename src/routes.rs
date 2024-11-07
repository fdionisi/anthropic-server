use std::{convert::Infallible, str::FromStr, sync::Arc};

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

use crate::{
    client::Client,
    provider::Provider,
    usage_reporter::{UsageReport, UsageReporter},
};

fn get_model(model: &AnthropicModel, provider: &Provider) -> Result<String> {
    Ok(match provider {
        Provider::Anthropic { .. } => model.to_string(),
        Provider::Bedrock { .. } => match model {
            AnthropicModel::ClaudeThreeDotFiveSonnet => BedrockModel::ClaudeThreeDotFiveSonnetV1,
            AnthropicModel::ClaudeThreeSonnet => BedrockModel::ClaudeThreeSonnet,
            AnthropicModel::ClaudeThreeOpus => BedrockModel::ClaudeThreeOpus,
            AnthropicModel::ClaudeThreeHaiku => BedrockModel::ClaudeThreeHaiku,
        }
        .to_string(),
        Provider::VertexAi { .. } => match model {
            AnthropicModel::ClaudeThreeDotFiveSonnet => VertexAiModel::ClaudeThreeDotFiveSonnetV1,
            AnthropicModel::ClaudeThreeSonnet => VertexAiModel::ClaudeThreeSonnet,
            AnthropicModel::ClaudeThreeOpus => VertexAiModel::ClaudeThreeOpus,
            AnthropicModel::ClaudeThreeHaiku => VertexAiModel::ClaudeThreeHaiku,
        }
        .to_string(),
    })
}

// XXX: This is required until Bedrock supports the same max_token as Anthropic or Vertex AI
fn get_max_tokens(model: &AnthropicModel, provider: &Provider, max_token: u32) -> u32 {
    match provider {
        Provider::Anthropic { .. } | Provider::VertexAi { .. } => {
            if matches!(model, AnthropicModel::ClaudeThreeDotFiveSonnet)
                || matches!(model, AnthropicModel::ClaudeThreeHaiku)
            {
                return max_token.min(8192);
            }
            max_token.min(4096)
        }
        Provider::Bedrock { .. } => max_token.min(4096),
    }
}

pub async fn healthz() -> Response {
    ((StatusCode::OK, Json(json!({ "status": "ok" })))).into_response()
}

pub async fn messages(
    State(client): State<Client>,
    State(provider): State<Provider>,
    State(usage_reporter): State<Arc<dyn UsageReporter>>,
    Json(request): Json<IncomingCreateMessageRequest>,
) -> Response {
    let IncomingCreateMessageRequest {
        stream,
        mut create_message_request,
    } = request;

    let model: AnthropicModel = match AnthropicModel::from_str(&create_message_request.model) {
        Ok(model) => model,
        Err(_) => {
            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(axum::body::Body::from(
                    json!({ "error": "Invalid model" }).to_string(),
                ))
                .unwrap()
        }
    };

    create_message_request.max_tokens =
        get_max_tokens(&model, &provider, create_message_request.max_tokens);

    create_message_request.model = match get_model(&model, &provider) {
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

        let stream = stream.map(move |item| {
            let item = item.unwrap();

            match item {
                anthropic::messages::Event::MessageDelta { ref usage, .. } => {
                    match usage_reporter.report(UsageReport {
                        model: model.to_string(),
                        usage: usage.clone(),
                    }) {
                        Err(err) => tracing::warn!(err = err.to_string(), "usage reporting failed"),
                        _ => {}
                    };
                }
                _ => {}
            }

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
                CreateMessageResponse::Message(ref message) => {
                    match usage_reporter.report(UsageReport {
                        model: model.to_string(),
                        usage: message.usage.clone(),
                    }) {
                        Err(err) => tracing::warn!(err = err.to_string(), "usage reporting failed"),
                        _ => {}
                    };

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
