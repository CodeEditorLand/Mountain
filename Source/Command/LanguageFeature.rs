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
//! ## Planned Work
//!
//! - Document symbols, formatting, rename, signature help
//! - Semantic tokens, code lens, inlay hints
//! - Linked editing range, call/type hierarchy
//! - Document color, folding/selection range
//! - Request dedupe and cancellation tokens for long-running ops

/// Mountainprovidecodeactions module.
pub mod MountainProvideCodeActions;

/// Mountainprovidecompletions module.
pub mod MountainProvideCompletions;

/// Mountainprovidedefinition module.
pub mod MountainProvideDefinition;

/// Mountainprovidedocumenthighlights module.
pub mod MountainProvideDocumentHighlights;

/// Mountainprovidehover module.
pub mod MountainProvideHover;

/// Mountainprovidereferences module.
pub mod MountainProvideReferences;

pub(crate) mod CodeActions;

pub(crate) mod Completions;

pub(crate) mod Definition;

pub(crate) mod Highlights;

pub(crate) mod Hover;

pub(crate) mod References;

pub(crate) mod InvokeProvider;

pub(crate) mod Validation;
