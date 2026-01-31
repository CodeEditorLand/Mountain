//! # RunTime
//!
//! Runtime subsystem for effect execution and lifecycle management.
//!
//! ## RESPONSIBILITIES
//!
//! ### Effect Execution
//! - Provide ApplicationRunTime for executing ActionEffect
//! - Bridge Echo scheduler with declarative effect system
//! - Support timeout and retry mechanisms for effect execution
//! - Handle effect cancellation and error recovery
//!
//! ### Lifecycle Management
//! - Orchestrate graceful shutdown of all services
//! - Coordinate service cleanup order (Cocoon, Terminals, State)
//! - Implement retry mechanisms for shutdown failures
//!
//! ## ARCHITECTURAL ROLE
//!
//! ### Position in Mountain
//! - Component of Core Infrastructure
//! - Implements ApplicationRunTime trait from Common
//! - Powered by Echo task scheduler
//!
//! ### Dependencies
//! - Echo::Scheduler: Task scheduling and execution
//! - Common::Effect::ApplicationRunTime: Trait implementation
//! - Common::Environment: Environment integration
//! - MountainEnvironment: Capability provider
//!
//! ### Dependents
//! - Binary: Initializes and manages ApplicationRunTime
//! - Command handlers: Submit effects for execution
//! - IPC services: Execute effects on behalf of frontend
//!
//! ## TODO
//!
//! ### Immediate Improvements
//! - Add effect execution metrics collection
//! - Implement effect prioritization system
//! - Add effect dependency tracking
//!
//! ### Future Work
//! - Implement effect result caching
//! - Add distributed effect execution
//! - Implement effect pipeline with chaining
//!
//! ### Missing Functionality to Probe
//! - Optimal timeout values for different effect types
//! - Retry strategy customization per effect
//! - Effect execution throttling under load

#![allow(non_snake_case, non_camel_case_types)]

pub mod ApplicationRunTime;
