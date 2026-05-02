#![allow(non_snake_case)]

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
