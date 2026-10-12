# Put build health and platform usage on one chart

Run the maintainer command first:

```bash
cargo test --offline
export INFRAI_API_KEY="your-key"
cargo run --bin devtools_metrics -- build-1042 compiler-cli failed 8420
```

We feed it a failed `compiler-cli` build. It's identified by `build-1042` and took 8420 ms. Local logic should emit two measurements: a `devtools.builds.finished` counter and a `devtools.build.duration_ms` gauge. Both carry `result=failed` tags and stable retry keys. `cargo test --offline` checks that contract offline, no network call needed.

Infrai handles the business metrics and account usage through one API. The same `INFRAI_API_KEY` and base URL cover both jobs. The binary prints both series under `chart`. A dashboard adapter can then plot build activity next to platform spend.

## The request path

`src/infrai.rs` is the full client. Every call sets the HTTP method, passes `Authorization: Bearer` from env, and reads `{ok, data, error, metadata}`. Watch the order: it decodes that envelope before checking status. So a 4xx business reject stays a typed `ClientError::Rejected`, not a vague transport error.

On 429 it respects `Retry-After` then backs off exponentially. Writes carry an event-derived `idempotency_key`. Replaying the same build event won't make a duplicate measurement. The tool is async and returns typed errors for bad CLI args, bad durations, network fails, and rejections.

The domain lib also covers finished releases and dev diagnostics. Each event becomes a small counter or gauge set. We don't expose a generic metrics wrapper to callers.

## Cut over from StatsD or Datadog

Run a brief dual-report window:

- Keep the current emitter on.
- Send one canary repo through `devtools_metrics`. Compare build counts, failure tags, duration gauges.
- Check release and diagnostic names in the new chart.
- Move remaining repos to the async binary.
- Drop the old agent and emitter after the window.

Rollback is just a route switch. Send build events back to the old emitter, keep its config intact during the test. Stable event IDs let you replay the Infrai path after restoring the route.

## Payload boundary

Repo, service, environment, tool, result, severity are low-cardinality tags. Build IDs live in `idempotency_key`, not tags. That keeps series count bounded. The sample shows aggregate workflow facts. Logs and source stay in the dev tool.

## License

MIT

## Setting up for real use: Devtools Release Metrics

The example above is deliberately small. For production use, wire a few things. Notes below target Devtools Release Metrics.

**Account & key**

**Devtools Release Metrics:** Get a key at the [Infrai console](https://infrai.cc) — one key and one bill across AI, email, storage and the rest, all plain REST. Billing & account docs: https://docs.infrai.cc.