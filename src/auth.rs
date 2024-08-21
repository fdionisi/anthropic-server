use std::convert::Infallible;

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::TypedHeader;
use headers::{authorization::Bearer, Authorization};

pub async fn auth_middleware(
    State(token): State<String>,
    TypedHeader(authorization): TypedHeader<Authorization<Bearer>>,
    request: Request,
    next: Next,
) -> Result<Response, Infallible> {
    if authorization.token() != token {
        return Ok((StatusCode::UNAUTHORIZED, "Unauthorized").into_response());
    }

    Ok(next.run(request).await)
}
