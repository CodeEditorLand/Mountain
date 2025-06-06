
// Defines all Data Transfer Objects (DTOs) related to language features.

#![allow(non_snake_case, non_camel_case_types)]

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::OptionsDto::SpecificProviderOptionsDto;

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "PascalCase")]
pub struct PositionDto {
	#[serde(alias = "lineNumber")]
	pub LineNumber:u32,
	pub Column:u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "PascalCase")]
pub struct RangeDto {
	#[serde(alias = "startLineNumber")]
	pub StartLineNumber:u32,
	#[serde(alias = "startColumn")]
	pub StartColumn:u32,
	#[serde(alias = "endLineNumber")]
	pub EndLineNumber:u32,
	#[serde(alias = "endColumn")]
	pub EndColumn:u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct IMarkdownStringDto {
	pub Value:String,
	#[serde(skip_serializing_if = "Option::is_none", alias = "isTrusted")]
	pub IsTrusted:Option<Value>,
	#[serde(skip_serializing_if = "Option::is_none", alias = "supportThemeIcons")]
	pub SupportThemeIcons:Option<bool>,
	#[serde(skip_serializing_if = "Option::is_none", alias = "supportHtml")]
	pub SupportHtml:Option<bool>,
	#[serde(skip_serializing_if = "Option::is_none", alias = "baseUri")]
	pub BaseUri:Option<Value>,
	#[serde(skip_serializing_if = "Option::is_none", alias = "uris")]
	pub UriMap:Option<HashMap<String, Value>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct HoverResultDto {
	pub Content:Vec<IMarkdownStringDto>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub Range:Option<RangeDto>,
}

pub type CompletionContextDto = Value;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct SuggestResultDto {
	#[serde(rename = "x", skip_serializing_if = "Option::is_none")]
	pub ListCacheIdentifier:Option<u32>,
	#[serde(rename = "b")]
	pub SuggestionList:Vec<Value>,
	#[serde(rename = "a")]
	pub DefaultRange:Value,
	#[serde(rename = "c", skip_serializing_if = "Option::is_none")]
	pub IsIncomplete:Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileEditTypeDto {
	Text = 1,
	File = 2,
	Cell = 3,
	CellReplace = 4,
	Snippet = 5,
	CellMetadata = 6,
	DocumentMetadata = 7,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct WorkspaceEditEntryBaseDto {
	#[serde(rename = "_type")]
	pub EditType:FileEditTypeDto,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub Metadata:Option<Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct WorkspaceTextEditDto {
	#[serde(flatten)]
	pub Base:WorkspaceEditEntryBaseDto,
	pub Resource:Value,
	pub Edit:Value,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub VersionIdentifier:Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct WorkspaceFileEditDto {
	#[serde(flatten)]
	pub Base:WorkspaceEditEntryBaseDto,
	#[serde(skip_serializing_if = "Option::is_none", alias = "oldUri")]
	pub OldUri:Option<Value>,
	#[serde(skip_serializing_if = "Option::is_none", alias = "newUri")]
	pub NewUri:Option<Value>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub Options:Option<Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct WorkspaceCellEditDto {
	#[serde(flatten)]
	pub Base:WorkspaceEditEntryBaseDto,
	pub Resource:Value,
	#[serde(alias = "cellEditPayload")]
	pub CellEditPayload:Value,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "PascalCase")]
pub struct WorkspaceEditDto {
	#[serde(alias = "edits")]
	pub EditList:Vec<Value>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub Metadata:Option<Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct RelatedInformationDto {
	pub Resource:Value,
	pub Message:String,
	#[serde(alias = "startLineNumber")]
	pub StartLineNumber:u32,
	#[serde(alias = "startColumn")]
	pub StartColumn:u32,
	#[serde(alias = "endLineNumber")]
	pub EndLineNumber:u32,
	#[serde(alias = "endColumn")]
	pub EndColumn:u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "PascalCase")]
pub struct MarkerDataDto {
	pub Severity:u32,
	pub Message:String,
	#[serde(alias = "startLineNumber")]
	pub StartLineNumber:u32,
	#[serde(alias = "startColumn")]
	pub StartColumn:u32,
	#[serde(alias = "endLineNumber")]
	pub EndLineNumber:u32,
	#[serde(alias = "endColumn")]
	pub EndColumn:u32,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub Source:Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub Code:Option<Value>,
	#[serde(skip_serializing_if = "Option::is_none")]
	#[serde(alias = "modelVersionId")]
	pub ModelVersionIdentifier:Option<u64>,
	#[serde(skip_serializing_if = "Option::is_none")]
	#[serde(alias = "relatedInformation")]
	pub RelatedInformation:Option<Vec<RelatedInformationDto>>,
	#[serde(skip_serializing_if = "Option::is_none")]
	#[serde(alias = "tags")]
	pub TagList:Option<Vec<u32>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct CommandDto {
	pub Identifier:String,
	pub Title:String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub Tooltip:Option<String>,
	#[serde(skip_serializing_if = "Option::is_none", alias = "arguments")]
	pub ArgumentList:Option<Vec<Value>>,
	#[serde(rename = "$ident", skip_serializing_if = "Option::is_none")]
	pub CacheIdentifier:Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct CodeActionDto {
	pub Title:String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub Kind:Option<String>,
	#[serde(skip_serializing_if = "Option::is_none", alias = "diagnostics")]
	pub DiagnosticList:Option<Vec<MarkerDataDto>>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub Edit:Option<WorkspaceEditDto>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub Command:Option<CommandDto>,
	#[serde(skip_serializing_if = "Option::is_none", alias = "isPreferred")]
	pub IsPreferred:Option<bool>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub Disabled:Option<String>,
	#[serde(skip_serializing_if = "Option::is_none", alias = "ranges")]
	pub RangeList:Option<Vec<RangeDto>>,
	#[serde(skip_serializing_if = "Option::is_none", alias = "isAi")]
	pub IsAi:Option<bool>,
	#[serde(skip_serializing_if = "Option::is_none", alias = "cacheId")]
	pub CacheIdentifier:Option<(u32, u32)>,
	#[serde(skip_serializing_if = "Option::is_none", alias = "_isSynthetic")]
	pub IsSynthetic:Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct CodeActionListDto {
	#[serde(alias = "actions")]
	pub ActionList:Vec<CodeActionDto>,
	#[serde(alias = "cacheId")]
	pub CacheIdentifier:u32,
}

pub type CodeActionContextDto = Value;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct CodeLensDto {
	pub Range:RangeDto,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub Command:Option<CommandDto>,
	#[serde(skip_serializing_if = "Option::is_none", alias = "cacheId")]
	pub CacheIdentifier:Option<(u32, u32)>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct CodeLensListDto {
	#[serde(alias = "lenses")]
	pub LensList:Vec<CodeLensDto>,
	#[serde(alias = "cacheId")]
	pub CacheIdentifier:u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct DocumentSymbolDto {
	pub Name:String,
	pub Detail:String,
	pub Kind:u32,
	#[serde(skip_serializing_if = "Option::is_none", alias = "tags")]
	pub TagList:Option<Vec<u32>>,
	pub Range:RangeDto,
	#[serde(alias = "selectionRange")]
	pub SelectionRange:RangeDto,
	#[serde(skip_serializing_if = "Option::is_none", alias = "children")]
	pub ChildList:Option<Vec<DocumentSymbolDto>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct WorkspaceSymbolDto {
	pub Name:String,
	pub Kind:u32,
	#[serde(skip_serializing_if = "Option::is_none", alias = "tags")]
	pub TagList:Option<Vec<u32>>,
	#[serde(skip_serializing_if = "Option::is_none", alias = "containerName")]
	pub ContainerName:Option<String>,
	pub Location:Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct ParameterInformationDto {
	pub Label:Value,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub Documentation:Option<IMarkdownStringDto>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct SignatureInformationDto {
	pub Label:String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub Documentation:Option<IMarkdownStringDto>,
	#[serde(skip_serializing_if = "Option::is_none", alias = "parameters")]
	pub ParameterList:Option<Vec<ParameterInformationDto>>,
	#[serde(skip_serializing_if = "Option::is_none", alias = "activeParameter")]
	pub ActiveParameter:Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct SignatureHelpResultDto {
	#[serde(alias = "signatures")]
	pub SignatureList:Vec<SignatureInformationDto>,
	#[serde(alias = "activeSignature")]
	pub ActiveSignature:u32,
	#[serde(alias = "activeParameter")]
	pub ActiveParameter:u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "PascalCase")]
pub enum SignatureHelpTriggerKindDto {
	Invoke = 1,
	TriggerCharacter = 2,
	ContentChange = 3,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct SignatureHelpContextDto {
	#[serde(alias = "triggerKind")]
	pub TriggerKind:SignatureHelpTriggerKindDto,
	#[serde(skip_serializing_if = "Option::is_none", alias = "triggerCharacter")]
	pub TriggerCharacter:Option<String>,
	#[serde(alias = "isRetrigger")]
	pub IsRetrigger:bool,
	#[serde(skip_serializing_if = "Option::is_none", alias = "activeSignatureHelp")]
	pub ActiveSignatureHelp:Option<SignatureHelpResultDto>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct TextEditDto {
	pub Range:RangeDto,
	pub Text:String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub Eol:Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct FormattingOptionsDto {
	#[serde(alias = "tabSize")]
	pub TabSize:u32,
	#[serde(alias = "insertSpaces")]
	pub InsertSpaces:bool,
	#[serde(flatten)]
	pub AdditionalPropertyMap:HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum DocumentHighlightKindDto {
	Text = 0,
	Read = 1,
	Write = 2,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct DocumentHighlightDto {
	pub Range:RangeDto,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub Kind:Option<DocumentHighlightKindDto>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct LinkDto {
	pub Range:RangeDto,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub Url:Option<Value>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub Tooltip:Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub Data:Option<Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct LinksListDto {
	#[serde(alias = "links")]
	pub LinkList:Vec<LinkDto>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct LocationLinkDto {
	#[serde(skip_serializing_if = "Option::is_none", alias = "originSelectionRange")]
	pub OriginSelectionRange:Option<RangeDto>,
	#[serde(alias = "targetUri")]
	pub TargetUri:Value,
	#[serde(alias = "targetRange")]
	pub TargetRange:RangeDto,
	#[serde(skip_serializing_if = "Option::is_none", alias = "targetSelectionRange")]
	pub TargetSelectionRange:Option<RangeDto>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct FoldingRangeDto {
	#[serde(alias = "startLine")]
	pub StartLine:u32,
	#[serde(alias = "endLine")]
	pub EndLine:u32,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub Kind:Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct SelectionRangeDto {
	pub Range:RangeDto,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub Parent:Option<Box<SelectionRangeDto>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct LinkedEditingRangesDto {
	#[serde(alias = "ranges")]
	pub RangeList:Vec<RangeDto>,
	#[serde(skip_serializing_if = "Option::is_none", alias = "wordPattern")]
	pub WordPattern:Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "PascalCase")]
pub struct SemanticTokensLegendDto {
	#[serde(alias = "tokenTypes")]
	pub TokenTypeList:Vec<String>,
	#[serde(alias = "tokenModifiers")]
	pub TokenModifierList:Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "PascalCase")]
pub struct SemanticTokensDto {
	#[serde(skip_serializing_if = "Option::is_none", alias = "resultId")]
	pub ResultIdentifier:Option<String>,
	pub Data:Vec<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct SemanticTokensEditDto {
	pub Start:u32,
	#[serde(alias = "deleteCount")]
	pub DeleteCount:u32,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub Data:Option<Vec<u32>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "PascalCase")]
pub struct SemanticTokensEditsDto {
	#[serde(skip_serializing_if = "Option::is_none", alias = "resultId")]
	pub ResultIdentifier:Option<String>,
	#[serde(alias = "edits")]
	pub EditList:Vec<SemanticTokensEditDto>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct HierarchyItemDto {
	#[serde(flatten)]
	pub SymbolInformation:DocumentSymbolDto,
	#[serde(alias = "_sessionId")]
	pub SessionIdentifier:String,
	#[serde(alias = "_itemId")]
	pub ItemIdentifier:String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct IncomingCallDto {
	pub From:HierarchyItemDto,
	#[serde(alias = "fromRanges")]
	pub FromRangeList:Vec<RangeDto>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct OutgoingCallDto {
	pub To:HierarchyItemDto,
	#[serde(alias = "fromRanges")]
	pub FromRangeList:Vec<RangeDto>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct InlayHintLabelPartDto {
	pub Value:String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub Tooltip:Option<Value>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub Location:Option<Value>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub Command:Option<CommandDto>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum InlayHintKindDto {
	Type = 1,
	Parameter = 2,
	Other = 0,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct InlayHintDto {
	pub Label:Value,
	pub Position:PositionDto,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub Kind:Option<InlayHintKindDto>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub Tooltip:Option<Value>,
	#[serde(skip_serializing_if = "Option::is_none", alias = "paddingLeft")]
	pub PaddingLeft:Option<bool>,
	#[serde(skip_serializing_if = "Option::is_none", alias = "paddingRight")]
	pub PaddingRight:Option<bool>,
	#[serde(skip_serializing_if = "Option::is_none", alias = "textEdits")]
	pub TextEditList:Option<Vec<TextEditDto>>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub Data:Option<Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProviderType {
	Hover,
	Completion,
	Definition,
	Declaration,
	Implementation,
	TypeDefinition,
	References,
	DocumentHighlight,
	DocumentSymbol,
	WorkspaceSymbol,
	CodeAction,
	CodeLens,
	Formatting,
	RangeFormatting,
	OnTypeFormatting,
	Rename,
	DocumentLink,
	Color,
	FoldingRange,
	SelectionRange,
	CallHierarchy,
	TypeHierarchy,
	LinkedEditingRange,
	InlayHints,
	SemanticTokens,
	SemanticTokensRange,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "PascalCase")]
pub struct ProviderOptionsDto {
	#[serde(skip_serializing_if = "Option::is_none", alias = "displayName")]
	pub DisplayName:Option<String>,
	// The rest of the fields were moved into the more specific `SpecificProviderOptionsDto` enum.
	// This is kept for backward compatibility if needed but the specific enum is preferred.
	#[serde(flatten)]
	pub AdditionalProperties:HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct ProviderDescription {
	pub Handle:u32,
	#[serde(alias = "sidecarId")]
	pub SidecarIdentifier:String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub Options:Option<Value>, // Can hold a serialized SpecificProviderOptionsDto
}
