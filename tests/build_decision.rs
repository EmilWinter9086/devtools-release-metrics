use devtools_release_metrics::{metrics_for, DevtoolsEvent};

#[test]
fn failed_build_emits_count_and_duration_with_stable_retry_keys() {
    let event = DevtoolsEvent::BuildFinished {
        repository: "compiler-cli".to_owned(),
        succeeded: false,
        duration_ms: 8_420,
    };

    let metrics = metrics_for("build-1042", &event);

    assert_eq!(metrics.len(), 2);
    assert_eq!(metrics[0].name, "devtools.builds.finished");
    assert_eq!(metrics[0].tags["result"], "failed");
    assert_eq!(metrics[0].idempotency_key, "build-1042:build-count");
    assert_eq!(metrics[1].name, "devtools.build.duration_ms");
    assert_eq!(metrics[1].value, 8_420.0);
    assert_eq!(metrics[1].idempotency_key, "build-1042:build-duration");
}

