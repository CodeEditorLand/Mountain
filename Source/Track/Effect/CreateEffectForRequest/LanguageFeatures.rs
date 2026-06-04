pub fn Matches(MethodName:&str) -> bool { MethodName.starts_with("register_") && MethodName.ends_with("_provider") }

use std::sync::Arc;

use CommonLibrary::{
	Environment::Requires::Requires,
	LanguageFeature::{
		DTO::ProviderType::ProviderType,
		LanguageFeatureProviderRegistry::LanguageFeatureProviderRegistry,
	},
};
use serde_json::{Value, json};
use tauri::Runtime;

use crate::Track::Effect::{
	CreateEffectForRequest::Utilities::Params::{obj_str, obj_val},
	MappedEffectType::MappedEffect,
};

fn CreateProviderEffect(Parameters:&Value, ProviderKind:ProviderType) -> Option<Result<MappedEffect, String>> {
	// Defer registration into the async effect so we clone once here.
	let id_need = Parameters.get("handle").and_then(Value::as_str).unwrap_or("").to_string();

	let selector_need = Parameters.get("language_selector").cloned().unwrap_or(Value::Null);

	let extension_id_need = Parameters.get("extension_id").cloned().unwrap_or(Value::Null);

	let options_need = Parameters.get("options").cloned();

	crate::effect!(run_time, {
		let provider:Arc<dyn LanguageFeatureProviderRegistry> = run_time.Environment.Require();

		let id = id_need;

		let selector = selector_need;

		let extension_id = extension_id_need;

		let options = options_need;

		provider
			.RegisterProvider(id, ProviderKind, selector, extension_id, options)
			.await
			.map(|handle| json!(handle))
			.map_err(|e| e.to_string())
	})
}

pub fn CreateEffect<R:Runtime>(MethodName:&str, Parameters:Value) -> Option<Result<MappedEffect, String>> {
	match MethodName {
		"register_hover_provider" => CreateProviderEffect(Parameters, ProviderType::Hover),

		"register_completion_item_provider" => CreateProviderEffect(Parameters, ProviderType::Completion),

		"register_definition_provider" => CreateProviderEffect(Parameters, ProviderType::Definition),

		"register_reference_provider" => CreateProviderEffect(Parameters, ProviderType::References),

		"register_code_actions_provider" => CreateProviderEffect(Parameters, ProviderType::CodeAction),

		"register_document_highlight_provider" => CreateProviderEffect(Parameters, ProviderType::DocumentHighlight),

		"register_document_symbol_provider" => CreateProviderEffect(Parameters, ProviderType::DocumentSymbol),

		"register_workspace_symbol_provider" => CreateProviderEffect(Parameters, ProviderType::WorkspaceSymbol),

		"register_rename_provider" => CreateProviderEffect(Parameters, ProviderType::Rename),

		"register_document_formatting_provider" => CreateProviderEffect(Parameters, ProviderType::DocumentFormatting),

		"register_document_range_formatting_provider" => {
			CreateProviderEffect(Parameters, ProviderType::DocumentRangeFormatting)
		},

		"register_on_type_formatting_provider" => CreateProviderEffect(Parameters, ProviderType::OnTypeFormatting),

		"register_signature_help_provider" => CreateProviderEffect(Parameters, ProviderType::SignatureHelp),

		"register_code_lens_provider" => CreateProviderEffect(Parameters, ProviderType::CodeLens),

		"register_folding_range_provider" => CreateProviderEffect(Parameters, ProviderType::FoldingRange),

		"register_selection_range_provider" => CreateProviderEffect(Parameters, ProviderType::SelectionRange),

		"register_semantic_tokens_provider" => CreateProviderEffect(Parameters, ProviderType::SemanticTokens),

		"register_inlay_hints_provider" => CreateProviderEffect(Parameters, ProviderType::InlayHint),

		"register_type_hierarchy_provider" => CreateProviderEffect(Parameters, ProviderType::TypeHierarchy),

		"register_call_hierarchy_provider" => CreateProviderEffect(Parameters, ProviderType::CallHierarchy),

		"register_linked_editing_range_provider" => CreateProviderEffect(Parameters, ProviderType::LinkedEditingRange),

		"register_document_link_provider" => CreateProviderEffect(Parameters, ProviderType::DocumentLink),

		"register_color_provider" => CreateProviderEffect(Parameters, ProviderType::Color),

		"register_implementation_provider" => CreateProviderEffect(Parameters, ProviderType::Implementation),

		"register_type_definition_provider" => CreateProviderEffect(Parameters, ProviderType::TypeDefinition),

		"register_declaration_provider" => CreateProviderEffect(Parameters, ProviderType::Declaration),

		"register_evaluatable_expression_provider" => {
			CreateProviderEffect(Parameters, ProviderType::EvaluatableExpression)
		},

		"register_inline_values_provider" => CreateProviderEffect(Parameters, ProviderType::InlineValues),

		// Providers added in VS Code ≥1.87 - registration wires the handle into
		// the ProviderRegistration map so the Language Feature dispatch layer
		// can proxy back to Cocoon for `$provideXxx` requests.
		"register_inline_completion_item_provider" => CreateProviderEffect(Parameters, ProviderType::InlineCompletion),

		"register_inline_edit_provider" => CreateProviderEffect(Parameters, ProviderType::InlineEdit),

		"register_multi_document_highlight_provider" => {
			CreateProviderEffect(Parameters, ProviderType::MultiDocumentHighlight)
		},

		"register_mapped_edits_provider" => CreateProviderEffect(Parameters, ProviderType::MappedEdits),

		"register_document_paste_edit_provider" => CreateProviderEffect(Parameters, ProviderType::DocumentPasteEdit),

		"register_document_drop_edit_provider" => CreateProviderEffect(Parameters, ProviderType::DocumentDropEdit),

		_ => None,
	}
}
