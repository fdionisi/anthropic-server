use axum::http::{HeaderName, HeaderValue};
use headers::Header;

pub static API_KEY_HEADER: &str = "x-api-key";

pub struct ApiKeyHeader(String);

impl ApiKeyHeader {
    pub fn key(&self) -> &str {
        &self.0
    }
}

impl Header for ApiKeyHeader {
    fn name() -> &'static HeaderName {
        static NAME: HeaderName = HeaderName::from_static(API_KEY_HEADER);
        &NAME
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, headers::Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i HeaderValue>,
    {
        let raw = values
            .next()
            .ok_or_else(headers::Error::invalid)?
            .to_str()
            .map_err(|_| headers::Error::invalid())?;

        Ok(Self(raw.to_string()))
    }

    fn encode<E: Extend<HeaderValue>>(&self, values: &mut E) {
        values.extend(std::iter::once(HeaderValue::from_str(&self.0).unwrap()));
    }
}
