//! Mountain RPC services. The active surface is `CocoonService` - the tonic
//! server impl that Cocoon dials into. The other modules here
//! (`EchoAction`, `Commands`, `Workspace`, `Configuration`, plus the
//! `Windows`/`Terminals`/`Debug`/`SCM`/`Processes`/`Telemetry` cfg-feature
//! shells) are scaffolding for the multi-extension-host roadmap; most carry
//! zero external callers today.

pub mod CocoonService;

pub mod Types;

pub mod EchoAction;

pub mod Commands;

pub mod Workspace;

pub mod Configuration;

#[cfg(any(feature = "grove", feature = "cocoon"))]
pub mod Windows;

#[cfg(feature = "terminals")]
pub mod Terminals;

#[cfg(feature = "debug-protocol")]
pub mod Debug;

#[cfg(feature = "scm-support")]
pub mod SCM;

#[cfg(feature = "child-processes")]
pub mod Processes;

pub mod Telemetry;

pub mod Vine;
