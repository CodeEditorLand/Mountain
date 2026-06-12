//! Language-feature-provider handlers for `CocoonService`. 44 entry points
//! split between `Register*` (21 files: hover/completion/definition/...,
//! the on-type-formatting / signature-help / semantic-tokens variants
//! that carry custom request shapes) and `Provide*` (23 files dispatching
//! the typed gRPC requests to the corresponding
//! `LanguageFeatureProviderRegistry` methods on the environment).
// --- Hierarchy prepare entry-points ---
// These establish the root item before incoming/outgoing/sub/supertypes.
/// PrepareCallHierarchy handler: prepares the root call-hierarchy item for a
/// symbol.
pub mod PrepareCallHierarchy;

/// PrepareTypeHierarchy handler: prepares the root type-hierarchy item for a
/// symbol.
pub mod PrepareTypeHierarchy;

/// ProvideCallHierarchyIncomingCalls handler: provides incoming call references
/// for a call-hierarchy item.
pub mod ProvideCallHierarchyIncomingCalls;

/// ProvideCallHierarchyOutgoingCalls handler: provides outgoing call references
/// for a call-hierarchy item.
pub mod ProvideCallHierarchyOutgoingCalls;

/// ProvideCodeActions handler: provides code actions at a given position.
pub mod ProvideCodeActions;

/// ProvideCodeLenses handler: provides code lenses for a document.
pub mod ProvideCodeLenses;

/// ProvideCompletionItems handler: provides completion items at a given
/// position.
pub mod ProvideCompletionItems;

/// ProvideDefinition handler: provides go-to-definition results for a symbol.
pub mod ProvideDefinition;

/// ProvideDocumentFormatting handler: provides full-document formatting edits.
pub mod ProvideDocumentFormatting;

/// ProvideDocumentHighlights handler: provides document highlights for a
/// symbol.
pub mod ProvideDocumentHighlights;

/// ProvideDocumentRangeFormatting handler: provides range-based formatting
/// edits.
pub mod ProvideDocumentRangeFormatting;

/// ProvideDocumentSymbols handler: provides the symbol tree for a document.
pub mod ProvideDocumentSymbols;

/// ProvideFoldingRanges handler: provides folding ranges for a document.
pub mod ProvideFoldingRanges;

/// ProvideHover handler: provides hover content for a symbol.
pub mod ProvideHover;

/// ProvideInlayHints handler: provides inlay hints for a document.
pub mod ProvideInlayHints;

/// ProvideInlineCompletionItems handler: provides inline completion
/// suggestions.
pub mod ProvideInlineCompletionItems;

/// ProvideLinkedEditingRanges handler: provides linked editing ranges for a
/// symbol.
pub mod ProvideLinkedEditingRanges;

/// ProvideOnTypeFormatting handler: provides on-type formatting edits.
pub mod ProvideOnTypeFormatting;

/// ProvideReferences handler: provides reference locations for a symbol.
pub mod ProvideReferences;

/// ProvideRenameEdits handler: provides rename edits for a symbol across files.
pub mod ProvideRenameEdits;

/// ProvideSelectionRanges handler: provides selection ranges at given
/// positions.
pub mod ProvideSelectionRanges;

/// ProvideSemanticTokensFull handler: provides full semantic token data for a
/// document.
pub mod ProvideSemanticTokensFull;

/// ProvideSignatureHelp handler: provides signature help for a function call.
pub mod ProvideSignatureHelp;

/// ProvideTypeHierarchySubtypes handler: provides subtypes for a type-hierarchy
/// item.
pub mod ProvideTypeHierarchySubtypes;

/// ProvideTypeHierarchySupertypes handler: provides supertypes for a
/// type-hierarchy item.
pub mod ProvideTypeHierarchySupertypes;

/// ProvideWorkspaceSymbols handler: provides workspace-wide symbol search
/// results.
pub mod ProvideWorkspaceSymbols;

/// RegisterCallHierarchyProvider handler: registers a call-hierarchy provider.
pub mod RegisterCallHierarchyProvider;

/// RegisterCodeActionsProvider handler: registers a code-action provider.
pub mod RegisterCodeActionsProvider;

/// RegisterCodeLensProvider handler: registers a code-lens provider.
pub mod RegisterCodeLensProvider;

/// RegisterCompletionItemProvider handler: registers a completion-item
/// provider.
pub mod RegisterCompletionItemProvider;

/// RegisterDefinitionProvider handler: registers a definition provider.
pub mod RegisterDefinitionProvider;

/// RegisterDocumentFormattingProvider handler: registers a document-formatting
/// provider.
pub mod RegisterDocumentFormattingProvider;

/// RegisterDocumentHighlightProvider handler: registers a document-highlight
/// provider.
pub mod RegisterDocumentHighlightProvider;

/// RegisterDocumentRangeFormattingProvider handler: registers a
/// range-formatting provider.
pub mod RegisterDocumentRangeFormattingProvider;

/// RegisterDocumentSymbolProvider handler: registers a document-symbol
/// provider.
pub mod RegisterDocumentSymbolProvider;

/// RegisterFoldingRangeProvider handler: registers a folding-range provider.
pub mod RegisterFoldingRangeProvider;

/// RegisterHoverProvider handler: registers a hover provider.
pub mod RegisterHoverProvider;

/// RegisterInlayHintsProvider handler: registers an inlay-hints provider.
pub mod RegisterInlayHintsProvider;

/// RegisterLinkedEditingRangeProvider handler: registers a linked-editing-range
/// provider.
pub mod RegisterLinkedEditingRangeProvider;

/// RegisterOnTypeFormattingProvider handler: registers an on-type-formatting
/// provider.
pub mod RegisterOnTypeFormattingProvider;

/// RegisterReferenceProvider handler: registers a reference provider.
pub mod RegisterReferenceProvider;

/// RegisterRenameProvider handler: registers a rename provider.
pub mod RegisterRenameProvider;

/// RegisterSelectionRangeProvider handler: registers a selection-range
/// provider.
pub mod RegisterSelectionRangeProvider;

/// RegisterSemanticTokensProvider handler: registers a semantic-tokens
/// provider.
pub mod RegisterSemanticTokensProvider;

/// RegisterSignatureHelpProvider handler: registers a signature-help provider.
pub mod RegisterSignatureHelpProvider;

/// RegisterTypeHierarchyProvider handler: registers a type-hierarchy provider.
pub mod RegisterTypeHierarchyProvider;

/// RegisterWorkspaceSymbolProvider handler: registers a workspace-symbol
/// provider.
pub mod RegisterWorkspaceSymbolProvider;
