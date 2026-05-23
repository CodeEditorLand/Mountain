//! Build-time PostHog credentials baked from `.env.Land.PostHog` via
//! `cargo:rustc-env`. `env!` always resolves at compile time so even a
//! clean checkout builds without a populated `.env`.

/// PostHog project token (`Authorize` in `.env.Land.PostHog`).
pub const POSTHOG_API_KEY:&str = env!("Authorize");

/// PostHog ingestion host (`Beam`). Defaults to EU Cloud; operators
/// override per environment.
pub const POSTHOG_HOST:&str = env!("Beam");

/// Per-tier enable flag (`Report`). String-comparison gate avoids
/// forking the binary per env value.
pub const POSTHOG_ENABLED:&str = env!("Report");

/// Optional pinned distinct-id seed (`Brand`). Empty → auto-generate
/// per process; populated → pinned across every process in the same
/// dev run for cross-restart correlation.
pub const POSTHOG_DISTINCT_ID_SEED:&str = env!("Brand");

/// OTLP collector endpoint (`OTLPEndpoint`). Default points at the local
/// Jaeger all-in-one HTTP receiver from `Land/Container/Compose.yaml`.
/// Read by `IPC::DevLog::EmitOTLPSpan` instead of the previous
/// hard-coded `127.0.0.1:4318`.
pub const OTLP_ENDPOINT:&str = env!("OTLPEndpoint");

/// Per-tier OTLP enable flag (`OTLPEnabled`). Mirrors `POSTHOG_ENABLED`
/// for the OTLP pipe. Independent of `Disable` so each pipe can be
/// flipped separately during diagnosis.
pub const OTLP_ENABLED:&str = env!("OTLPEnabled");

/// Master telemetry kill switch (`Capture`). `false` short-circuits BOTH
/// PostHog and OTLP regardless of their per-pipe toggles. Distinct from
/// `.env.Land.Diagnostics`'s `Disable` which kills polyfills/shims.
pub const TELEMETRY_CAPTURE:&str = env!("Capture");

/// Span / element filter (`Trace`). RUST_LOG-shaped string consumed by
/// the `IPC::DevLog::IsEnabled::Fn` filter. `all` enables every tag.
/// Reserved for the eventual `tracing-subscriber` EnvFilter; currently
/// duplicated with `IPC::DevLog::IsEnabled` which reads `Trace` at
/// runtime via `std::env::var`.
pub const TRACE_FILTER:&str = env!("Trace");
