// File: Track/SkyCommandDto.rs
// Defines the Data Transfer Object (DTO) structures for commands originating
// from the Sky frontend. These structs are used to deserialize the `args`
// `Value` from a Tauri command into a strongly-typed Rust struct.

#![allow(non_snake_case, non_camel_case_types)]

use Common::LanguageFeatureEffect::{
	CodeActionContextDto,
	CompletionContextDto,
	FormattingOptionsDto,
	HierarchyItemDto,
	PositionDto,
	RangeDto,
	SignatureHelpContextDto,
	WorkspaceEditDto,
};
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct RequestHoverArgument {
	#[serde(alias = "uriString")]
	pub UriString:String,
	#[serde(alias = "lineNumber0Based")]
	pub LineNumberZeroBased:u32,
	#[serde(alias = "column0Based")]
	pub ColumnZeroBased:u32,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct RequestCompletionsArgument {
	#[serde(alias = "uriString")]
	pub UriString:String,
	#[serde(alias = "lineNumber0Based")]
	pub LineNumberZeroBased:u32,
	#[serde(alias = "column0Based")]
	pub ColumnZeroBased:u32,
	#[serde(alias = "contextDto")]
	pub ContextDto:CompletionContextDto,
	#[serde(alias = "cancellationTokenIdVal")]
	pub CancellationTokenIdentifierValue:Option<Value>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ResolveCompletionItemArgument {
	#[serde(alias = "listCacheId")]
	pub ListCacheIdentifier:u32,
	#[serde(alias = "itemDtoToResolve")]
	pub ItemDtoToResolve:Value,
	#[serde(alias = "cancellationTokenIdVal")]
	pub CancellationTokenIdentifierValue:Option<Value>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct RequestCodeActionsArgument {
	#[serde(alias = "uriString")]
	pub UriString:String,
	#[serde(alias = "lineNumber0BasedStart")]
	pub LineNumberZeroBasedStart:u32,
	#[serde(alias = "column0BasedStart")]
	pub ColumnZeroBasedStart:u32,
	#[serde(alias = "lineNumber0BasedEnd")]
	pub LineNumberZeroBasedEnd:u32,
	#[serde(alias = "column0BasedEnd")]
	pub ColumnZeroBasedEnd:u32,
	#[serde(alias = "contextDto")]
	pub ContextDto:CodeActionContextDto,
	#[serde(alias = "cancellationTokenIdVal")]
	pub CancellationTokenIdentifierValue:Option<Value>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ResolveCodeActionArgument {
	#[serde(alias = "listCacheId")]
	pub ListCacheIdentifier:u32,
	#[serde(alias = "actionDtoToResolve")]
	pub ActionDtoToResolve:Value,
	#[serde(alias = "cancellationTokenIdVal")]
	pub CancellationTokenIdentifierValue:Option<Value>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct RequestCodeLensesArgument {
	#[serde(alias = "uriString")]
	pub UriString:String,
	#[serde(alias = "cancellationTokenIdVal")]
	pub CancellationTokenIdentifierValue:Option<Value>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ResolveCodeLensArgument {
	#[serde(alias = "listCacheId")]
	pub ListCacheIdentifier:u32,
	#[serde(alias = "lensDtoToResolve")]
	pub LensDtoToResolve:Value,
	#[serde(alias = "cancellationTokenIdVal")]
	pub CancellationTokenIdentifierValue:Option<Value>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct DocumentSymbolsArgument {
	#[serde(alias = "uriString")]
	pub UriString:String,
	#[serde(alias = "cancellationTokenIdVal")]
	pub CancellationTokenIdentifierValue:Option<Value>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct WorkspaceSymbolsArgument {
	pub Query:String,
	#[serde(alias = "cancellationTokenIdVal")]
	pub CancellationTokenIdentifierValue:Option<Value>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct SignatureHelpArgument {
	#[serde(alias = "uriString")]
	pub UriString:String,
	#[serde(alias = "lineNumber0Based")]
	pub LineNumberZeroBased:u32,
	#[serde(alias = "column0Based")]
	pub ColumnZeroBased:u32,
	#[serde(alias = "contextDto")]
	pub ContextDto:SignatureHelpContextDto,
	#[serde(alias = "cancellationTokenIdVal")]
	pub CancellationTokenIdentifierValue:Option<Value>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct RequestReferencesArgument {
	#[serde(alias = "uriString")]
	pub UriString:String,
	#[serde(alias = "lineNumber0Based")]
	pub LineNumberZeroBased:u32,
	#[serde(alias = "column0Based")]
	pub ColumnZeroBased:u32,
	#[serde(alias = "contextDto")]
	pub ContextDto:Value,
	#[serde(alias = "cancellationTokenIdVal")]
	pub CancellationTokenIdentifierValue:Option<Value>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct PrepareRenameArgument {
	#[serde(alias = "uriString")]
	pub UriString:String,
	#[serde(alias = "lineNumber0Based")]
	pub LineNumberZeroBased:u32,
	#[serde(alias = "column0Based")]
	pub ColumnZeroBased:u32,
	#[serde(alias = "cancellationTokenIdVal")]
	pub CancellationTokenIdentifierValue:Option<Value>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ProvideRenameEditsArgument {
	#[serde(alias = "uriString")]
	pub UriString:String,
	#[serde(alias = "lineNumber0Based")]
	pub LineNumberZeroBased:u32,
	#[serde(alias = "column0Based")]
	pub ColumnZeroBased:u32,
	#[serde(alias = "newName")]
	pub NewName:String,
	#[serde(alias = "cancellationTokenIdVal")]
	pub CancellationTokenIdentifierValue:Option<Value>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct FormattingArgument {
	#[serde(alias = "uriString")]
	pub UriString:String,
	#[serde(alias = "optionsDto")]
	pub OptionsDto:FormattingOptionsDto,
	#[serde(alias = "tokenVal")]
	pub TokenValue:Option<Value>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct PositionalArgument {
	#[serde(alias = "uriString")]
	pub UriString:String,
	pub Line:u32,
	pub Character:u32,
	#[serde(alias = "tokenVal")]
	pub TokenValue:Option<Value>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct DocumentArgument {
	#[serde(alias = "uriString")]
	pub UriString:String,
	#[serde(alias = "tokenVal")]
	pub TokenValue:Option<Value>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ResolveLinkArgument {
	#[serde(alias = "linkDtoVal")]
	pub LinkDtoValue:Value,
	#[serde(alias = "tokenVal")]
	pub TokenValue:Option<Value>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct FoldingRangesArgument {
	#[serde(alias = "uriString")]
	pub UriString:String,
	#[serde(alias = "contextDto")]
	pub ContextDto:Value,
	#[serde(alias = "tokenVal")]
	pub TokenValue:Option<Value>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct SelectionRangesArgument {
	#[serde(alias = "uriString")]
	pub UriString:String,
	#[serde(alias = "positionsDto")]
	pub PositionsDto:Vec<PositionDto>,
	#[serde(alias = "tokenVal")]
	pub TokenValue:Option<Value>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct LinkedEditingArgument {
	#[serde(alias = "uriString")]
	pub UriString:String,
	pub Line:u32,
	pub Character:u32,
	#[serde(alias = "tokenVal")]
	pub TokenValue:Option<Value>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct SemanticTokensArgument {
	#[serde(alias = "uriString")]
	pub UriString:String,
	#[serde(alias = "previousResultId")]
	pub PreviousResultIdentifier:Option<String>,
	#[serde(alias = "tokenVal")]
	pub TokenValue:Option<Value>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct SemanticTokensEditsArgument {
	#[serde(alias = "uriString")]
	pub UriString:String,
	#[serde(alias = "previousResultId")]
	pub PreviousResultIdentifier:String,
	#[serde(alias = "tokenVal")]
	pub TokenValue:Option<Value>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct PrepareHierarchyArgument {
	#[serde(alias = "uriString")]
	pub UriString:String,
	pub Line:u32,
	pub Character:u32,
	#[serde(alias = "tokenVal")]
	pub TokenValue:Option<Value>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ProvideHierarchyDetailArgument {
	#[serde(alias = "itemDto")]
	pub ItemDto:HierarchyItemDto,
	#[serde(alias = "tokenVal")]
	pub TokenValue:Option<Value>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct RequestInlayHintsArgument {
	#[serde(alias = "uriString")]
	pub UriString:String,
	#[serde(alias = "startLine")]
	pub StartLine:u32,
	#[serde(alias = "startChar")]
	pub StartCharacter:u32,
	#[serde(alias = "endLine")]
	pub EndLine:u32,
	#[serde(alias = "endChar")]
	pub EndCharacter:u32,
	#[serde(alias = "tokenVal")]
	pub TokenValue:Option<Value>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ResolveInlayHintArgument {
	#[serde(alias = "hintDtoToResolveVal")]
	pub HintDtoToResolveValue:Value,
	#[serde(alias = "tokenVal")]
	pub TokenValue:Option<Value>,
}
