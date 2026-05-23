
//! # Connection pool
//!
//! Bounded IPC connection pool with health monitoring,
//! lifetime / idle cleanup, and statistics. The
//! `Pool::Struct` aggregator + giant impl lives in `Pool.rs`
//! (tightly-coupled cluster); the per-connection state, the
//! health-status enum, the config, the stats DTO, and the
//! background health checker each live in their own sibling.

pub mod ConnectionHandle;

pub mod ConnectionHealth;

pub mod HealthChecker;

pub mod Pool;

pub mod PoolConfig;

pub mod PoolStats;
