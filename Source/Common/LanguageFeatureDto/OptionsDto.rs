
// Defines Data Transfer Objects (DTOs) for provider-specific options
// that are sent during language feature provider registration.

#![allow(non_snake_case, non_camel_case_types)]

use serde::{Deserialize, Serialize};

use crate::LanguageFeatureEffect::SemanticTokensLegendDto;

/// DTO for completion provider options.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct CompletionOptionsDto {
	#[serde(alias = "triggerCharacters")]
	pub TriggerCharacterList:Vec<String>,
	#[serde(alias = "supportsResolveDetails")]
	pub SupportsResolveDetails:bool,
}

/// DTO for signature help provider options.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct SignatureHelpOptionsDto {
	#[serde(alias = "triggerCharacters")]
	pub TriggerCharacterList:Vec<String>,
	#[serde(alias = "retriggerCharacters")]
	pub RetriggerCharacterList:Vec<String>,
}

/// DTO for code action provider metadata.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct CodeActionProviderMetadataInternalDto {
	#[serde(alias = "providedCodeActionKinds")]
	pub ProvidedCodeActionKindList:Option<Vec<String>>,
}

/// DTO for semantic tokens provider options.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct SemanticTokensProviderOptionsDto {
	pub Legend:SemanticTokensLegendDto,
	#[serde(alias = "documentSupportsEdits")]
	pub DocumentSupportsEdits:Option<bool>,
	#[serde(alias = "rangeSupportsEdits")]
	pub RangeSupportsEdits:Option<bool>,
}

/// An enum that wraps various provider-specific option DTOs.
/// This allows for type-safe handling of different provider configurations.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub enum SpecificProviderOptionsDto {
	None,
	Common {
		#[serde(alias = "displayName")]
		DisplayName:Option<String>,
	},
	Completion(CompletionOptionsDto),
	SignatureHelp(SignatureHelpOptionsDto),
	CodeAction(CodeActionProviderMetadataInternalDto),
	SemanticTokens(SemanticTokensProviderOptionsDto),
	CodeLens {
		#[serde(alias = "onDidChangeCodeLensesEventHandle")]
		OnChangeCodeLensesEventHandle:Option<u32>,
	},
	InlayHints {
		#[serde(alias = "inlayHintsSupportsResolve")]
		InlayHintsSupportResolve:Option<bool>,
		#[serde(alias = "onDidChangeInlayHintsEventHandle")]
		OnChangeInlayHintsEventHandle:Option<u32>,
	},
	FoldingRange {
		#[serde(alias = "onDidChangeFoldingRangesEventHandle")]
		OnChangeFoldingRangesEventHandle:Option<u32>,
	},
}
