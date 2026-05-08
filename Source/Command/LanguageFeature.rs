#![allow(non_snake_case)]

//! # LanguageFeature (Tauri command surface)
//!
//! Bridges Monaco-Editor language requests from Sky into the Mountain
//! `LanguageFeatureProvider` registry. Six wire-bound commands, each in
//! its own file (file name = Tauri command identifier per the
//! Naming-Convention exception):
//!
//! - `MountainProvideHover::MountainProvideHover`
//! - `MountainProvideCodeActions::MountainProvideCodeActions`
//! - `MountainProvideDocumentHighlights::MountainProvideDocumentHighlights`
//! - `MountainProvideCompletions::MountainProvideCompletions`
//! - `MountainProvideDefinition::MountainProvideDefinition`
//! - `MountainProvideReferences::MountainProvideReferences`
//!
//! Each command is a thin shell that validates input and delegates to
//! the matching `provide_*_impl` in the sibling `Hover` / `CodeActions`
//! / … modules. Implementation files are `pub(crate)` because callers
//! outside this directory should go through the wire-bound shells.
//!
//! Errors propagate as `Result<Value, String>` with the string sent
//! straight to the frontend; provider errors (`CommonError`) are
//! stringified at the boundary.
//!
//! VS Code reference: `vs/workbench/api/common/extHostLanguageFeatures.ts`,
//! `vs/workbench/services/languageFeatures/common/languageFeaturesService.ts`.
//!
//! TODO: document symbols, formatting, rename, signature help, semantic
//! tokens, code lens, inlay hints, linked editing range, call/type
//! hierarchy, document color, folding/selection range, request dedupe
//! and cancellation tokens for long-running ops.

pub mod MountainProvideCodeActions;

pub mod MountainProvideCompletions;

pub mod MountainProvideDefinition;

pub mod MountainProvideDocumentHighlights;

pub mod MountainProvideHover;

pub mod MountainProvideReferences;

pub(crate) mod CodeActions;

pub(crate) mod Completions;

pub(crate) mod Definition;

pub(crate) mod Highlights;

pub(crate) mod Hover;

pub(crate) mod References;

pub(crate) mod InvokeProvider;

pub(crate) mod Validation;
