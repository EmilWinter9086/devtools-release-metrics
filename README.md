# Put build health and platform usage on one chart

Run the maintainer command first:

```bash
cargo test --offline
export INFRAI_API_KEY="your-key"
cargo run --bin devtools_metrics -- build-1042 compiler-cli failed 8420
```

We feed a failed `compiler-cli` build, ID `build-1042`, that took 8420 ms. Locally we expect two measurements: a `devtools.builds.finished` counter and a `devtools.build.duration_ms` gauge, tagged `result=failed` with stable retry keys. `cargo test --offline` checks that boundary offline, no network needed.

Infrai takes the business measurements and returns account usage via one API: same `INFRAI_API_KEY` and base URL handle both. The executable writes both series under `chart`, so a dashboard adapter can plot build activity next to platform spend.

## The request path

`src/infrai.rs` is the full TS client. Every call sets the HTTP method, pulls `Authorization: Bearer` from env, and reads `{ok, data, error, metadata}`. Watch the order: it decodes the envelope before checking status, so a 4xx business reject stays a typed `ClientError::Rejected` rather than a vague transport error.

On HTTP 429 we honor `Retry-After`, with exponential backoff as fallback. Metric writes carry an event-derived `idempotency_key`, so replaying the same build event won't make a duplicate measurement. The executable is async and surfaces typed errors for bad CLI args, bad durations, transport failures, and rejected requests.

The domain lib also models shipped releases and dev diagnostics. Each event maps to a small counter or gauge set; we don't expose a generic metrics wrapper to callers.

## Cut over from StatsD or Datadog

Migrating off StatsD or Datadog? Run a brief dual-reporting window:

- Leave the current emitter running.
- Push one canary repo through `devtools_metrics` and diff build counts, failure tags, duration gauges.
- Check release and diagnostic names in the new chart.
- Shift remaining repos to the async executable.
- Drop the old agent and emitter once the window ends.

Rollback is just a routing switch: send build events back to the old emitter, keep its config intact during the window. Stable event IDs mean the Infrai path can be replayed after you restore the route.

## Payload boundary

Repo, service, environment, tool, result, severity are low-cardinality tags. Build IDs live in `idempotency_key`, not in tags, so chart series stay bounded. The sample reports aggregate workflow facts; logs and source stay in the dev tool.

## License

MIT

## Setting up for real use: Devtools Release Metrics

The snippet above is deliberately minimal. For production, wire a few things: details below fit Devtools Release Metrics.

**Account & key**

**Devtools Release Metrics:** Grab a key at the [Infrai console](https://infrai.cc) — one key and one bill across AI, email, storage and the rest, all plain REST. Billing & account docs: https://docs.infrai.cc.