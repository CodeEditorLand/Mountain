
//! Language-feature-provider handlers for `CocoonService`. 44 entry points
//! split between `Register*` (21 files: hover/completion/definition/...,
//! the on-type-formatting / signature-help / semantic-tokens variants
//! that carry custom request shapes) and `Provide*` (23 files dispatching
//! the typed gRPC requests to the corresponding
//! `LanguageFeatureProviderRegistry` methods on the environment).

// --- Hierarchy prepare entry-points ---
// These establish the root item before incoming/outgoing/sub/supertypes.
pub mod PrepareCallHierarchy;

pub mod PrepareTypeHierarchy;

pub mod ProvideCallHierarchyIncomingCalls;

pub mod ProvideCallHierarchyOutgoingCalls;

pub mod ProvideCodeActions;

pub mod ProvideCodeLenses;

pub mod ProvideCompletionItems;

pub mod ProvideDefinition;

pub mod ProvideDocumentFormatting;

pub mod ProvideDocumentHighlights;

pub mod ProvideDocumentRangeFormatting;

pub mod ProvideDocumentSymbols;

pub mod ProvideFoldingRanges;

pub mod ProvideHover;

pub mod ProvideInlayHints;

pub mod ProvideInlineCompletionItems;

pub mod ProvideLinkedEditingRanges;

pub mod ProvideOnTypeFormatting;

pub mod ProvideReferences;

pub mod ProvideRenameEdits;

pub mod ProvideSelectionRanges;

pub mod ProvideSemanticTokensFull;

pub mod ProvideSignatureHelp;

pub mod ProvideTypeHierarchySubtypes;

pub mod ProvideTypeHierarchySupertypes;

pub mod ProvideWorkspaceSymbols;

pub mod RegisterCallHierarchyProvider;

pub mod RegisterCodeActionsProvider;

pub mod RegisterCodeLensProvider;

pub mod RegisterCompletionItemProvider;

pub mod RegisterDefinitionProvider;

pub mod RegisterDocumentFormattingProvider;

pub mod RegisterDocumentHighlightProvider;

pub mod RegisterDocumentRangeFormattingProvider;

pub mod RegisterDocumentSymbolProvider;

pub mod RegisterFoldingRangeProvider;

pub mod RegisterHoverProvider;

pub mod RegisterInlayHintsProvider;

pub mod RegisterLinkedEditingRangeProvider;

pub mod RegisterOnTypeFormattingProvider;

pub mod RegisterReferenceProvider;

pub mod RegisterRenameProvider;

pub mod RegisterSelectionRangeProvider;

pub mod RegisterSemanticTokensProvider;

pub mod RegisterSignatureHelpProvider;

pub mod RegisterTypeHierarchyProvider;

pub mod RegisterWorkspaceSymbolProvider;
