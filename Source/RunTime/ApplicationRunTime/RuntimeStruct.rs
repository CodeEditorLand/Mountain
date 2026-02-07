//! # ApplicationRunTime (RunTime::ApplicationRunTime)
//!
//! ## RESPONSIBILITIES
//!
//! Defines the main ApplicationRunTime struct that provides effect execution
//! and lifecycle management capabilities for the Mountain application.
//!
//! ## ARCHITECTURAL ROLE
//!
//! Core execution engine in Mountain's architecture that bridges declarative
//! effect system with high-performance task execution through Echo scheduler.
//!
//! ## KEY COMPONENTS
//!
//! - **Scheduler**: Shared handle to Echo scheduler for task execution
//! - **Environment**: Shared handle to MountainEnvironment for capability access
//!
//! ## ERROR HANDLING
//!
//! Uses Result types for fallible operations. The struct itself is infallible to create
//! (all fields are Arc handles), but operations may return errors.
//!
//! ## LOGGING
//!
//! Uses log crate with appropriate severity levels for lifecycle events.
//!
//! ## PERFORMANCE CONSIDERATIONS
//!
//! - Uses Arc for shared state to minimize cloning
//! - Struct derives Clone for cheap duplication
//! - All fields are thread-safe Arc references
//!
//! ## TODO
//!
//! None

use std::sync::Arc;

use CommonLibrary::{
	Environment::{
		Environment::Environment,
		HasEnvironment::HasEnvironment,
	},
};
use Echo::Scheduler::Scheduler::Scheduler;

use crate::Environment::MountainEnvironment::MountainEnvironment;

/// A `RunTime` that uses a high-performance, work-stealing scheduler (`Echo`)
/// to execute all `ActionEffect`s.
#[derive(Clone)]
pub struct ApplicationRunTime {
	/// A shared handle to the application's central scheduler.
	pub Scheduler:Arc<Scheduler>,

	/// A shared handle to the application's `Environment`, providing all
	/// necessary capabilities.
	pub Environment:Arc<MountainEnvironment>,
}

impl ApplicationRunTime {
	/// Creates a new `ApplicationRunTime` that is powered by an `Echo`
	/// scheduler.
	pub fn Create(Scheduler:Arc<Scheduler>, Environment:Arc<MountainEnvironment>) -> Self {
		log::info!("[ApplicationRunTime] New Echo-based instance created.");

		Self { Scheduler, Environment }
	}
}

// Implement the marker trait to satisfy the bounds on ApplicationRunTimeTrait
impl HasEnvironment for ApplicationRunTime {
	type EnvironmentType = MountainEnvironment;

	fn GetEnvironment(&self) -> Arc<Self::EnvironmentType> { self.Environment.clone() }
}

// The ApplicationRunTime is not an environment itself, but it needs this marker
// to satisfy some complex generic bounds in the effect system.
impl Environment for ApplicationRunTime {}
