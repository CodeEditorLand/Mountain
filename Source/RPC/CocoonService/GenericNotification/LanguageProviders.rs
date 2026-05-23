
//! Notification handlers: `register_*_provider` language feature registrations.
//! All follow the same pattern: extract `handle`, `language_selector`,
//! `extension_id` from `Params` and call `RegisterProvider`.

use CommonLibrary::LanguageFeature::DTO::ProviderType::ProviderType;
use serde_json::Value;

use crate::RPC::CocoonService::CocoonServiceImpl;

pub fn HandleRegisterProvider(Params:Value, Service:&CocoonServiceImpl, ProvType:ProviderType) {
	let Handle = Params.get("handle").and_then(|V| V.as_u64()).unwrap_or(0) as u32;

	let Selector = Params.get("language_selector").and_then(|V| V.as_str()).unwrap_or("*");

	let ExtId = Params.get("extension_id").and_then(|V| V.as_str()).unwrap_or("");

	Service.RegisterProvider(Handle, ProvType, Selector, ExtId);
}

/// Dispatch a `register_*_provider` method string to the correct
/// `ProviderType`. Returns `true` if the method was recognised and dispatched.
pub fn DispatchRegisterProvider(Method:&str, Params:Value, Service:&CocoonServiceImpl) -> bool {
	let ProvType = match Method {
		"register_hover_provider" => ProviderType::Hover,

		"register_completion_item_provider" => ProviderType::Completion,

		"register_definition_provider" => ProviderType::Definition,

		"register_reference_provider" => ProviderType::References,

		"register_code_actions_provider" => ProviderType::CodeAction,

		"register_document_highlight_provider" => ProviderType::DocumentHighlight,

		"register_document_symbol_provider" => ProviderType::DocumentSymbol,

		"register_workspace_symbol_provider" => ProviderType::WorkspaceSymbol,

		"register_rename_provider" => ProviderType::Rename,

		"register_document_formatting_provider" => ProviderType::DocumentFormatting,

		"register_document_range_formatting_provider" => ProviderType::DocumentRangeFormatting,

		"register_on_type_formatting_provider" => ProviderType::OnTypeFormatting,

		"register_signature_help_provider" => ProviderType::SignatureHelp,

		"register_code_lens_provider" => ProviderType::CodeLens,

		"register_folding_range_provider" => ProviderType::FoldingRange,

		"register_selection_range_provider" => ProviderType::SelectionRange,

		"register_semantic_tokens_provider" => ProviderType::SemanticTokens,

		"register_inlay_hints_provider" => ProviderType::InlayHint,

		"register_type_hierarchy_provider" => ProviderType::TypeHierarchy,

		"register_call_hierarchy_provider" => ProviderType::CallHierarchy,

		"register_linked_editing_range_provider" => ProviderType::LinkedEditingRange,

		"register_document_link_provider" => ProviderType::DocumentLink,

		"register_color_provider" => ProviderType::Color,

		"register_implementation_provider" => ProviderType::Implementation,

		"register_type_definition_provider" => ProviderType::TypeDefinition,

		"register_declaration_provider" => ProviderType::Declaration,

		"register_evaluatable_expression_provider" => ProviderType::EvaluatableExpression,

		"register_inline_values_provider" => ProviderType::InlineValues,

		// Inline completion: GitHub Copilot, Roo Code, Continue, etc.
		"register_inline_completion_item_provider" => ProviderType::InlineCompletion,

		_ => return false,
	};

	HandleRegisterProvider(Params, Service, ProvType);

	true
}
