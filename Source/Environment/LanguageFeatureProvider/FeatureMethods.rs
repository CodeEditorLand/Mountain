//! All LSP feature method entry points. Each operation lives in its own
//! module under `FeatureMethods/`; the functions here are thin delegators
//! that keep the call sites in the trait impl (`mod.rs`) stable.

/// Invoke provider registration and dispatch.
#[path = "FeatureMethods/InvokeProvider.rs"]
pub mod InvokeProvider;

/// Invoke provider method registration and dispatch.
#[path = "FeatureMethods/InvokeProviderMethod.rs"]
pub mod InvokeProviderMethod;

/// Call hierarchy preparation registration and dispatch.
#[path = "FeatureMethods/PrepareCallHierarchy.rs"]
pub mod PrepareCallHierarchy;

/// Rename preparation registration and dispatch.
#[path = "FeatureMethods/PrepareRename.rs"]
pub mod PrepareRename;

/// Type hierarchy preparation registration and dispatch.
#[path = "FeatureMethods/PrepareTypeHierarchy.rs"]
pub mod PrepareTypeHierarchy;

/// Call hierarchy incoming calls provider registration and dispatch.
#[path = "FeatureMethods/ProvideCallHierarchyIncomingCalls.rs"]
pub mod ProvideCallHierarchyIncomingCalls;

/// Call hierarchy outgoing calls provider registration and dispatch.
#[path = "FeatureMethods/ProvideCallHierarchyOutgoingCalls.rs"]
pub mod ProvideCallHierarchyOutgoingCalls;

/// Code actions provider registration and dispatch.
#[path = "FeatureMethods/ProvideCodeActions.rs"]
pub mod ProvideCodeActions;

/// Code lenses provider registration and dispatch.
#[path = "FeatureMethods/ProvideCodeLenses.rs"]
pub mod ProvideCodeLenses;

/// Completions provider registration and dispatch.
#[path = "FeatureMethods/ProvideCompletions.rs"]
pub mod ProvideCompletions;

/// Go-to-definition provider registration and dispatch.
#[path = "FeatureMethods/ProvideDefinition.rs"]
pub mod ProvideDefinition;

/// Document formatting edits provider registration and dispatch.
#[path = "FeatureMethods/ProvideDocumentFormattingEdits.rs"]
pub mod ProvideDocumentFormattingEdits;

/// Document highlights provider registration and dispatch.
#[path = "FeatureMethods/ProvideDocumentHighlights.rs"]
pub mod ProvideDocumentHighlights;

/// Document links provider registration and dispatch.
#[path = "FeatureMethods/ProvideDocumentLinks.rs"]
pub mod ProvideDocumentLinks;

/// Document range formatting edits provider registration and dispatch.
#[path = "FeatureMethods/ProvideDocumentRangeFormattingEdits.rs"]
pub mod ProvideDocumentRangeFormattingEdits;

/// Document symbols provider registration and dispatch.
#[path = "FeatureMethods/ProvideDocumentSymbols.rs"]
pub mod ProvideDocumentSymbols;

/// File decoration provider registration and dispatch.
#[path = "FeatureMethods/ProvideFileDecoration.rs"]
pub mod ProvideFileDecoration;

/// Folding ranges provider registration and dispatch.
#[path = "FeatureMethods/ProvideFoldingRanges.rs"]
pub mod ProvideFoldingRanges;

/// Hover provider registration and dispatch.
#[path = "FeatureMethods/ProvideHover.rs"]
pub mod ProvideHover;

/// Inlay hints provider registration and dispatch.
#[path = "FeatureMethods/ProvideInlayHints.rs"]
pub mod ProvideInlayHints;

/// Inline completion items provider registration and dispatch.
#[path = "FeatureMethods/ProvideInlineCompletionItems.rs"]
pub mod ProvideInlineCompletionItems;

/// Linked editing ranges provider registration and dispatch.
#[path = "FeatureMethods/ProvideLinkedEditingRanges.rs"]
pub mod ProvideLinkedEditingRanges;

/// On-type formatting edits provider registration and dispatch.
#[path = "FeatureMethods/ProvideOnTypeFormattingEdits.rs"]
pub mod ProvideOnTypeFormattingEdits;

/// References provider registration and dispatch.
#[path = "FeatureMethods/ProvideReferences.rs"]
pub mod ProvideReferences;

/// Rename edits provider registration and dispatch.
#[path = "FeatureMethods/ProvideRenameEdits.rs"]
pub mod ProvideRenameEdits;

/// Selection ranges provider registration and dispatch.
#[path = "FeatureMethods/ProvideSelectionRanges.rs"]
pub mod ProvideSelectionRanges;

/// Semantic tokens full provider registration and dispatch.
#[path = "FeatureMethods/ProvideSemanticTokensFull.rs"]
pub mod ProvideSemanticTokensFull;

/// Signature help provider registration and dispatch.
#[path = "FeatureMethods/ProvideSignatureHelp.rs"]
pub mod ProvideSignatureHelp;

/// Type hierarchy subtypes provider registration and dispatch.
#[path = "FeatureMethods/ProvideTypeHierarchySubtypes.rs"]
pub mod ProvideTypeHierarchySubtypes;

/// Type hierarchy supertypes provider registration and dispatch.
#[path = "FeatureMethods/ProvideTypeHierarchySupertypes.rs"]
pub mod ProvideTypeHierarchySupertypes;

/// Workspace symbols provider registration and dispatch.
#[path = "FeatureMethods/ProvideWorkspaceSymbols.rs"]
pub mod ProvideWorkspaceSymbols;

use CommonLibrary::{
	Error::CommonError::CommonError,
	LanguageFeature::DTO::{
		CompletionContextDTO::CompletionContextDTO,
		CompletionListDTO::CompletionListDTO,
		HoverResultDTO::HoverResultDTO,
		LocationDTO::LocationDTO,
		PositionDTO::PositionDTO,
		TextEditDTO::TextEditDTO,
	},
};
use serde_json::Value;
use url::Url;

pub(super) async fn provide_code_actions(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	document_uri:Url,

	range_or_selection_dto:Value,

	context_dto:Value,
) -> Result<Option<Value>, CommonError> {
	ProvideCodeActions::Fn(environment, document_uri, range_or_selection_dto, context_dto).await
}

pub(super) async fn provide_code_lenses(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	document_uri:Url,
) -> Result<Option<Value>, CommonError> {
	ProvideCodeLenses::Fn(environment, document_uri).await
}

pub(super) async fn provide_completions(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	document_uri:Url,

	position_dto:PositionDTO,

	context_dto:CompletionContextDTO,

	cancellation_token_value:Option<Value>,
) -> Result<Option<CompletionListDTO>, CommonError> {
	ProvideCompletions::Fn(environment, document_uri, position_dto, context_dto, cancellation_token_value).await
}

pub(super) async fn provide_definition(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	document_uri:Url,

	position_dto:PositionDTO,
) -> Result<Option<Vec<LocationDTO>>, CommonError> {
	ProvideDefinition::Fn(environment, document_uri, position_dto).await
}

pub(super) async fn provide_document_formatting_edits(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	document_uri:Url,

	options_dto:Value,
) -> Result<Option<Vec<TextEditDTO>>, CommonError> {
	ProvideDocumentFormattingEdits::Fn(environment, document_uri, options_dto).await
}

pub(super) async fn provide_document_highlights(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	document_uri:Url,

	position_dto:PositionDTO,
) -> Result<Option<Value>, CommonError> {
	ProvideDocumentHighlights::Fn(environment, document_uri, position_dto).await
}

pub(super) async fn provide_document_links(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	document_uri:Url,
) -> Result<Option<Value>, CommonError> {
	ProvideDocumentLinks::Fn(environment, document_uri).await
}

pub(super) async fn provide_document_range_formatting_edits(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	document_uri:Url,

	range_dto:Value,

	options_dto:Value,
) -> Result<Option<Vec<TextEditDTO>>, CommonError> {
	ProvideDocumentRangeFormattingEdits::Fn(environment, document_uri, range_dto, options_dto).await
}

pub(super) async fn provide_hover(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	document_uri:Url,

	position_dto:PositionDTO,
) -> Result<Option<HoverResultDTO>, CommonError> {
	ProvideHover::Fn(environment, document_uri, position_dto).await
}

pub(super) async fn provide_references(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	document_uri:Url,

	position_dto:PositionDTO,

	context_dto:Value,
) -> Result<Option<Vec<LocationDTO>>, CommonError> {
	ProvideReferences::Fn(environment, document_uri, position_dto, context_dto).await
}

pub(super) async fn prepare_rename(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	document_uri:Url,

	position_dto:PositionDTO,
) -> Result<Option<Value>, CommonError> {
	PrepareRename::Fn(environment, document_uri, position_dto).await
}

pub(super) async fn provide_rename_edits(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	document_uri:Url,

	position_dto:PositionDTO,

	new_name:String,
) -> Result<Option<Value>, CommonError> {
	ProvideRenameEdits::Fn(environment, document_uri, position_dto, new_name).await
}

pub(super) async fn provide_document_symbols(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	document_uri:Url,
) -> Result<Option<Value>, CommonError> {
	ProvideDocumentSymbols::Fn(environment, document_uri).await
}

pub(super) async fn provide_workspace_symbols(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	query:String,
) -> Result<Option<Value>, CommonError> {
	ProvideWorkspaceSymbols::Fn(environment, query).await
}

pub(super) async fn provide_signature_help(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	document_uri:Url,

	position_dto:PositionDTO,

	context_dto:Value,
) -> Result<Option<Value>, CommonError> {
	ProvideSignatureHelp::Fn(environment, document_uri, position_dto, context_dto).await
}

pub(super) async fn provide_folding_ranges(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	document_uri:Url,
) -> Result<Option<Value>, CommonError> {
	ProvideFoldingRanges::Fn(environment, document_uri).await
}

pub(super) async fn provide_selection_ranges(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	document_uri:Url,

	positions:Vec<PositionDTO>,
) -> Result<Option<Value>, CommonError> {
	ProvideSelectionRanges::Fn(environment, document_uri, positions).await
}

pub(super) async fn provide_semantic_tokens_full(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	document_uri:Url,
) -> Result<Option<Value>, CommonError> {
	ProvideSemanticTokensFull::Fn(environment, document_uri).await
}

pub(super) async fn provide_inlay_hints(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	document_uri:Url,

	range_dto:Value,
) -> Result<Option<Value>, CommonError> {
	ProvideInlayHints::Fn(environment, document_uri, range_dto).await
}

pub(super) async fn provide_type_hierarchy_supertypes(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	item_dto:Value,
) -> Result<Option<Value>, CommonError> {
	ProvideTypeHierarchySupertypes::Fn(environment, item_dto).await
}

pub(super) async fn provide_type_hierarchy_subtypes(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	item_dto:Value,
) -> Result<Option<Value>, CommonError> {
	ProvideTypeHierarchySubtypes::Fn(environment, item_dto).await
}

pub(super) async fn prepare_call_hierarchy(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	document_uri:Url,

	position_dto:PositionDTO,
) -> Result<Option<Value>, CommonError> {
	PrepareCallHierarchy::Fn(environment, document_uri, position_dto).await
}

pub(super) async fn prepare_type_hierarchy(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	document_uri:Url,

	position_dto:PositionDTO,
) -> Result<Option<Value>, CommonError> {
	PrepareTypeHierarchy::Fn(environment, document_uri, position_dto).await
}

pub(super) async fn provide_call_hierarchy_incoming_calls(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	item_dto:Value,
) -> Result<Option<Value>, CommonError> {
	ProvideCallHierarchyIncomingCalls::Fn(environment, item_dto).await
}

pub(super) async fn provide_call_hierarchy_outgoing_calls(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	item_dto:Value,
) -> Result<Option<Value>, CommonError> {
	ProvideCallHierarchyOutgoingCalls::Fn(environment, item_dto).await
}

pub(super) async fn provide_linked_editing_ranges(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	document_uri:Url,

	position_dto:PositionDTO,
) -> Result<Option<Value>, CommonError> {
	ProvideLinkedEditingRanges::Fn(environment, document_uri, position_dto).await
}

pub(super) async fn provide_on_type_formatting_edits(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	document_uri:Url,

	position_dto:PositionDTO,

	character:String,

	options_dto:Value,
) -> Result<Option<Vec<TextEditDTO>>, CommonError> {
	ProvideOnTypeFormattingEdits::Fn(environment, document_uri, position_dto, character, options_dto).await
}

pub(super) async fn provide_file_decoration(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	resource_uri:Url,
) -> Result<Option<Value>, CommonError> {
	ProvideFileDecoration::Fn(environment, resource_uri).await
}

pub(super) async fn provide_inline_completion_items(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	document_uri:Url,

	position_dto:PositionDTO,

	context_dto:Value,
) -> Result<Option<Value>, CommonError> {
	ProvideInlineCompletionItems::Fn(environment, document_uri, position_dto, context_dto).await
}
