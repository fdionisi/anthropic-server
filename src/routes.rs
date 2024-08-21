use std::convert::Infallible;

use anthropic::messages::{CreateMessageRequestWithStream, Messages, MessagesStream};
use axum::{
    extract::State,
    response::{sse::Event, IntoResponse, Response, Sse},
    Json,
};
use futures::StreamExt;

use crate::client::Client;

pub async fn messages(
    State(client): State<Client>,
    Json(request): Json<CreateMessageRequestWithStream>,
) -> Response {
    if request.stream {
        let stream = client
            .messages_stream(request.create_message_request)
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
        Json(
            client
                .messages(request.create_message_request)
                .await
                .unwrap(),
        )
        .into_response()
    }
}
