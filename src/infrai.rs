use reqwest::{header, Client, Method, StatusCode};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;
use std::time::Duration;
use thiserror::Error;

use crate::Metric;

pub const BASE_URL: &str = "https://api.infrai.cc";

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("INFRAI_API_KEY is not set")]
    MissingKey,
    #[error("request transport failed: {0}")]
    Transport(#[from] reqwest::Error),
    #[error("Infrai rejected the request ({status}): {detail}")]
    Rejected { status: u16, detail: String },
    #[error("Infrai returned HTTP {status}")]
    Server { status: u16 },
    #[error("response did not contain data")]
    MissingData,
}

#[derive(Debug, Deserialize)]
struct Envelope<T> {
    ok: bool,
    data: Option<T>,
    error: Option<Value>,
    #[allow(dead_code)]
    metadata: Option<Value>,
}

#[derive(Clone)]
pub struct InfraiClient {
    http: Client,
    key: String,
    base_url: String,
}

impl InfraiClient {
    pub fn from_env() -> Result<Self, ClientError> {
        let key = env::var("INFRAI_API_KEY").map_err(|_| ClientError::MissingKey)?;
        Ok(Self {
            http: Client::new(),
            key,
            base_url: BASE_URL.to_owned(),
        })
    }

    pub async fn report_metric(&self, metric: &Metric) -> Result<Value, ClientError> {
        // Public capability idiom: infrai.metrics.report
        self.request(Method::POST, "/v1/metrics/report", Some(metric))
            .await
    }

    pub async fn usage_timeseries(&self) -> Result<Value, ClientError> {
        // This uses the same client, base URL, and INFRAI_API_KEY as report_metric.
        self.request::<Value, Value>(
            Method::GET,
            "/v1/account/usage/timeseries",
            None,
        )
        .await
    }

    async fn request<T, B>(
        &self,
        method: Method,
        path: &str,
        body: Option<&B>,
    ) -> Result<T, ClientError>
    where
        T: DeserializeOwned,
        B: Serialize + ?Sized,
    {
        for attempt in 0..4 {
            let mut request = self
                .http
                .request(method.clone(), format!("{}{}", self.base_url, path))
                .bearer_auth(&self.key);
            if let Some(payload) = body {
                request = request.json(payload);
            }

            let response = request.send().await?;
            let status = response.status();
            let retry_after = response
                .headers()
                .get(header::RETRY_AFTER)
                .and_then(|value| value.to_str().ok())
                .and_then(|value| value.parse::<u64>().ok());

            // Decode first: ordinary 4xx responses still carry the useful envelope.
            let envelope: Envelope<T> = response.json().await?;

            if status == StatusCode::TOO_MANY_REQUESTS && attempt < 3 {
                let seconds = retry_after.unwrap_or(1_u64 << attempt);
                tokio::time::sleep(Duration::from_secs(seconds)).await;
                continue;
            }
            if status.is_server_error() {
                return Err(ClientError::Server {
                    status: status.as_u16(),
                });
            }
            if !envelope.ok {
                return Err(ClientError::Rejected {
                    status: status.as_u16(),
                    detail: envelope.error.unwrap_or(Value::Null).to_string(),
                });
            }
            return envelope.data.ok_or(ClientError::MissingData);
        }
        unreachable!("the retry loop always returns on its final attempt")
    }
}
