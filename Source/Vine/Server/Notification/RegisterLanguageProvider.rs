#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! Handles all `register_*` / `register_*_provider` gRPC notifications from
//! the Cocoon extension host. Each such notification wires a language-feature
//! provider into Mountain's `ProviderRegistration` keyed on `Handle`; the
//! language-feature RPC path (e.g. `GetHoverAtPosition`) then proxies back to
//! Cocoon with the original `$providerXxx` method.
//!
//! Wire-method naming uses snake_case with two trailing shapes:
//! - plain verbs:     `register_rename`, `register_debug_adapter`
//! - `_provider` suffix: `register_hover_provider`,
//!   `register_code_lens_provider`
//!
//! Both forms are normalised by stripping `register_` prefix and optional
//! `_provider` suffix before the enum lookup.

use CommonLibrary::LanguageFeature::DTO::ProviderType::ProviderType as PT;
use serde_json::{Value, json};

use crate::{
	ApplicationState::DTO::ProviderRegistrationDTO::ProviderRegistrationDTO,
	Vine::Server::MountainVinegRPCService,
	dev_log,
};

/// Dispatch a `register_*` notification. Returns `true` if the method was
/// recognised and a `ProviderRegistrationDTO` was inserted.
pub async fn RegisterLanguageProvider(Service:&MountainVinegRPCService, MethodName:&str, Parameter:&Value) -> bool {
	let Handle = Parameter.get("handle").and_then(|h| h.as_u64()).unwrap_or(0) as u32;

	// Accept camelCase (current Cocoon shape) with snake_case fallback for
	// partial rebuild compatibility.
	let Selector = Parameter
		.get("languageSelector")
		.or_else(|| Parameter.get("language_selector"))
		.and_then(|s| s.as_str())
		.unwrap_or("*");
	let ExtId = Parameter
		.get("extensionId")
		.or_else(|| Parameter.get("extension_id"))
		.and_then(|e| e.as_str())
		.unwrap_or("");
	let Scheme = Parameter.get("scheme").and_then(|s| s.as_str()).unwrap_or("");

	let ProviderTypeName = MethodName
		.strip_prefix("register_")
		.map(|Stripped| Stripped.strip_suffix("_provider").unwrap_or(Stripped))
		.unwrap_or("");

	dev_log!(
		"grpc-verbose",
		"[MountainVinegRPCService] Cocoon registered {} provider: handle={}, lang={}",
		ProviderTypeName,
		Handle,
		Selector
	);
	dev_log!(
		"provider-register",
		"[ProviderRegister] accepted method={} type={} handle={} lang={} scheme={} ext={}",
		MethodName,
		ProviderTypeName,
		Handle,
		Selector,
		Scheme,
		ExtId
	);

	let ProvType:Option<PT> = match ProviderTypeName {
		"authentication" => Some(PT::Authentication),
		"call_hierarchy" => Some(PT::CallHierarchy),
		"code_actions" => Some(PT::CodeAction),
		"code_lens" => Some(PT::CodeLens),
		"color" => Some(PT::Color),
		"completion_item" => Some(PT::Completion),
		"debug_adapter" => Some(PT::DebugAdapter),
		"debug_configuration" => Some(PT::DebugConfiguration),
		"declaration" => Some(PT::Declaration),
		"definition" => Some(PT::Definition),
		"document_drop_edit" => Some(PT::DocumentDropEdit),
		"document_formatting" => Some(PT::DocumentFormatting),
		"document_highlight" => Some(PT::DocumentHighlight),
		"document_link" => Some(PT::DocumentLink),
		"document_paste_edit" => Some(PT::DocumentPasteEdit),
		"document_range_formatting" => Some(PT::DocumentRangeFormatting),
		"document_symbol" => Some(PT::DocumentSymbol),
		"evaluatable_expression" => Some(PT::EvaluatableExpression),
		"external_uri_opener" => Some(PT::ExternalUriOpener),
		"file_decoration" => Some(PT::FileDecoration),
		"file_system" => Some(PT::FileSystem),
		"folding_range" => Some(PT::FoldingRange),
		"hover" => Some(PT::Hover),
		"implementation" => Some(PT::Implementation),
		"inlay_hints" => Some(PT::InlayHint),
		"inline_completion_item" => Some(PT::InlineCompletion),
		"inline_edit" => Some(PT::InlineEdit),
		"inline_values" => Some(PT::InlineValues),
		"linked_editing_range" => Some(PT::LinkedEditingRange),
		"mapped_edits" => Some(PT::MappedEdits),
		"multi_document_highlight" => Some(PT::MultiDocumentHighlight),
		"notebook_content" => Some(PT::NotebookContent),
		"notebook_serializer" => Some(PT::NotebookSerializer),
		"on_type_formatting" => Some(PT::OnTypeFormatting),
		"reference" => Some(PT::References),
		"remote_authority_resolver" => Some(PT::RemoteAuthorityResolver),
		"rename" => Some(PT::Rename),
		"resource_label_formatter" => Some(PT::ResourceLabelFormatter),
		"scm" => Some(PT::SourceControl),
		"scm_resource_group" => Some(PT::ScmResourceGroup),
		"selection_range" => Some(PT::SelectionRange),
		"semantic_tokens" => Some(PT::SemanticTokens),
		"signature_help" => Some(PT::SignatureHelp),
		"task" => Some(PT::Task),
		"terminal_link" => Some(PT::TerminalLink),
		"terminal_profile" => Some(PT::TerminalProfile),
		"text_document_content" => Some(PT::TextDocumentContent),
		"type_definition" => Some(PT::TypeDefinition),
		"type_hierarchy" => Some(PT::TypeHierarchy),
		"uri_handler" => Some(PT::UriHandler),
		"workspace_symbol" => Some(PT::WorkspaceSymbol),
		_ => None,
	};

	let Some(ProviderType) = ProvType else { return false };

	// Scheme-bound providers carry their scheme in the selector so Mountain's
	// resolver (FileSystem router, URI handler dispatch, …) can match on it.
	let SelectorValue = if !Scheme.is_empty() {
		json!([{ "scheme": Scheme, "language": Selector }])
	} else {
		json!([{ "language": Selector }])
	};

	let Dto = ProviderRegistrationDTO {
		Handle,
		ProviderType,
		Selector:SelectorValue,
		SideCarIdentifier:"cocoon-main".to_string(),
		ExtensionIdentifier:json!(ExtId),
		Options:Parameter.get("options").cloned(),
	};

	Service
		.RunTime
		.Environment
		.ApplicationState
		.Extension
		.ProviderRegistration
		.RegisterProvider(Handle, Dto);

	true
}
