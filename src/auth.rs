use std::convert::Infallible;

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::TypedHeader;

use crate::api_key_header::ApiKeyHeader;

pub async fn auth_middleware(
    State(token): State<String>,
    TypedHeader(authorization): TypedHeader<ApiKeyHeader>,
    request: Request,
    next: Next,
) -> Result<Response, Infallible> {
    if authorization.key() != token {
        return Ok((StatusCode::UNAUTHORIZED, "Unauthorized").into_response());
    }

    Ok(next.run(request).await)
}
