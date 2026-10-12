use devtools_release_metrics::infrai::{ClientError, InfraiClient};
use devtools_release_metrics::{metrics_for, DevtoolsEvent};
use serde_json::{json, Value};
use std::env;
use thiserror::Error;

#[derive(Debug, Error)]
enum ServiceError {
    #[error("usage: devtools-metrics <event-id> <repository> <succeeded|failed> <duration-ms>")]
    Arguments,
    #[error("duration-ms must be an unsigned integer")]
    Duration,
    #[error(transparent)]
    Infrai(#[from] ClientError),
}

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("{error}");
        std::process::exit(2);
    }
}

async fn run() -> Result<(), ServiceError> {
    let mut args = env::args().skip(1);
    let event_id = args.next().ok_or(ServiceError::Arguments)?;
    let repository = args.next().ok_or(ServiceError::Arguments)?;
    let succeeded = match args.next().as_deref() {
        Some("succeeded") => true,
        Some("failed") => false,
        _ => return Err(ServiceError::Arguments),
    };
    let duration_ms = args
        .next()
        .ok_or(ServiceError::Arguments)?
        .parse()
        .map_err(|_| ServiceError::Duration)?;
    if args.next().is_some() {
        return Err(ServiceError::Arguments);
    }

    let event = DevtoolsEvent::BuildFinished {
        repository,
        succeeded,
        duration_ms,
    };
    let metrics = metrics_for(&event_id, &event);
    let infrai = InfraiClient::from_env()?;
    let mut business = Vec::<Value>::new();
    for metric in &metrics {
        infrai.report_metric(metric).await?;
        business.push(json!({"name": metric.name, "value": metric.value, "tags": metric.tags}));
    }
    let platform_usage = infrai.usage_timeseries().await?;

    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "chart": {
                "business_metrics": business,
                "platform_usage": platform_usage
            }
        }))
        .expect("chart values are serializable")
    );
    Ok(())
}

