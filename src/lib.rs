pub mod infrai;

use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DevtoolsEvent {
    BuildFinished {
        repository: String,
        succeeded: bool,
        duration_ms: u64,
    },
    ReleaseFinished {
        service: String,
        environment: String,
        succeeded: bool,
    },
    DiagnosticPublished {
        tool: String,
        severity: Severity,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Metric {
    pub r#type: &'static str,
    pub name: &'static str,
    pub value: f64,
    pub tags: serde_json::Value,
    pub idempotency_key: String,
}

pub fn metrics_for(event_id: &str, event: &DevtoolsEvent) -> Vec<Metric> {
    match event {
        DevtoolsEvent::BuildFinished {
            repository,
            succeeded,
            duration_ms,
        } => vec![
            Metric {
                r#type: "counter",
                name: "devtools.builds.finished",
                value: 1.0,
                tags: serde_json::json!({"repository": repository, "result": result(*succeeded)}),
                idempotency_key: format!("{event_id}:build-count"),
            },
            Metric {
                r#type: "gauge",
                name: "devtools.build.duration_ms",
                value: *duration_ms as f64,
                tags: serde_json::json!({"repository": repository, "result": result(*succeeded)}),
                idempotency_key: format!("{event_id}:build-duration"),
            },
        ],
        DevtoolsEvent::ReleaseFinished {
            service,
            environment,
            succeeded,
        } => vec![Metric {
            r#type: "counter",
            name: "devtools.releases.finished",
            value: 1.0,
            tags: serde_json::json!({
                "service": service,
                "environment": environment,
                "result": result(*succeeded)
            }),
            idempotency_key: format!("{event_id}:release-count"),
        }],
        DevtoolsEvent::DiagnosticPublished { tool, severity } => vec![Metric {
            r#type: "counter",
            name: "devtools.diagnostics.published",
            value: 1.0,
            tags: serde_json::json!({"tool": tool, "severity": severity_name(*severity)}),
            idempotency_key: format!("{event_id}:diagnostic-count"),
        }],
    }
}

fn result(succeeded: bool) -> &'static str {
    if succeeded { "succeeded" } else { "failed" }
}

fn severity_name(severity: Severity) -> &'static str {
    match severity {
        Severity::Info => "info",
        Severity::Warning => "warning",
        Severity::Error => "error",
    }
}

