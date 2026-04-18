#![allow(non_snake_case)]
//! Language Feature Provider domain handlers for CocoonService.
//!
//! Typed gRPC RPCs: all register_*_provider and provide_* methods for
//! language features (hover, completion, definition, references,
//! code_actions, document_highlights, document_symbols, workspace_symbols,
//! rename, document_formatting, document_range_formatting,
//! on_type_formatting, signature_help, code_lenses, folding_ranges,
//! selection_ranges, semantic_tokens, inlay_hints, type_hierarchy,
//! call_hierarchy, linked_editing_ranges).

use CommonLibrary::LanguageFeature::{
	DTO::{PositionDTO::PositionDTO, ProviderType::ProviderType},
	LanguageFeatureProviderRegistry::LanguageFeatureProviderRegistry,
};
use serde_json::json;
use tonic::{Response, Status};
use url::Url;

use super::CocoonServiceImpl;
use crate::{
	Vine::Generated::{
		CompletionItem,
		Empty,
		Location,
		Position,
		ProvideCallHierarchyRequest,
		ProvideCallHierarchyResponse,
		ProvideCodeActionsRequest,
		ProvideCodeActionsResponse,
		ProvideCodeLensesRequest,
		ProvideCodeLensesResponse,
		ProvideCompletionItemsRequest,
		ProvideCompletionItemsResponse,
		ProvideDefinitionRequest,
		ProvideDefinitionResponse,
		ProvideDocumentFormattingRequest,
		ProvideDocumentFormattingResponse,
		ProvideDocumentHighlightsRequest,
		ProvideDocumentHighlightsResponse,
		ProvideDocumentRangeFormattingRequest,
		ProvideDocumentRangeFormattingResponse,
		ProvideDocumentSymbolsRequest,
		ProvideDocumentSymbolsResponse,
		ProvideFoldingRangesRequest,
		ProvideFoldingRangesResponse,
		ProvideHoverRequest,
		ProvideHoverResponse,
		ProvideInlayHintsRequest,
		ProvideInlayHintsResponse,
		ProvideLinkedEditingRangesRequest,
		ProvideLinkedEditingRangesResponse,
		ProvideOnTypeFormattingRequest,
		ProvideOnTypeFormattingResponse,
		ProvideReferencesRequest,
		ProvideReferencesResponse,
		ProvideRenameEditsRequest,
		ProvideRenameEditsResponse,
		ProvideSelectionRangesRequest,
		ProvideSelectionRangesResponse,
		ProvideSemanticTokensRequest,
		ProvideSemanticTokensResponse,
		ProvideSignatureHelpRequest,
		ProvideSignatureHelpResponse,
		ProvideTypeHierarchyRequest,
		ProvideTypeHierarchyResponse,
		ProvideWorkspaceSymbolsRequest,
		ProvideWorkspaceSymbolsResponse,
		Range,
		RegisterOnTypeFormattingProviderRequest,
		RegisterProviderRequest,
		RegisterSemanticTokensProviderRequest,
		RegisterSignatureHelpProviderRequest,
		Uri,
	},
	dev_log,
};

// ==================== Registration Helpers ====================

pub async fn RegisterHoverProvider(
	Service:&CocoonServiceImpl,
	req:RegisterProviderRequest,
) -> Result<Response<Empty>, Status> {
	dev_log!(
		"cocoon",
		"[CocoonService] Registering hover provider for '{}' with handle {}",
		req.language_selector,
		req.handle
	);
	Service.RegisterProvider(req.handle, ProviderType::Hover, &req.language_selector, &req.extension_id);
	Ok(Response::new(Empty {}))
}

pub async fn RegisterCompletionItemProvider(
	Service:&CocoonServiceImpl,
	req:RegisterProviderRequest,
) -> Result<Response<Empty>, Status> {
	dev_log!(
		"cocoon",
		"[CocoonService] Registering completion provider for '{}' with handle {}",
		req.language_selector,
		req.handle
	);
	Service.RegisterProvider(req.handle, ProviderType::Completion, &req.language_selector, &req.extension_id);
	Ok(Response::new(Empty {}))
}

pub async fn RegisterDefinitionProvider(
	Service:&CocoonServiceImpl,
	req:RegisterProviderRequest,
) -> Result<Response<Empty>, Status> {
	dev_log!(
		"cocoon",
		"[CocoonService] Registering definition provider for '{}' with handle {}",
		req.language_selector,
		req.handle
	);
	Service.RegisterProvider(req.handle, ProviderType::Definition, &req.language_selector, &req.extension_id);
	Ok(Response::new(Empty {}))
}

pub async fn RegisterReferenceProvider(
	Service:&CocoonServiceImpl,
	req:RegisterProviderRequest,
) -> Result<Response<Empty>, Status> {
	dev_log!(
		"cocoon",
		"[CocoonService] Registering reference provider for '{}' with handle {}",
		req.language_selector,
		req.handle
	);
	Service.RegisterProvider(req.handle, ProviderType::References, &req.language_selector, &req.extension_id);
	Ok(Response::new(Empty {}))
}

pub async fn RegisterCodeActionsProvider(
	Service:&CocoonServiceImpl,
	req:RegisterProviderRequest,
) -> Result<Response<Empty>, Status> {
	dev_log!(
		"cocoon",
		"[CocoonService] Registering code actions provider for '{}' with handle {}",
		req.language_selector,
		req.handle
	);
	Service.RegisterProvider(req.handle, ProviderType::CodeAction, &req.language_selector, &req.extension_id);
	Ok(Response::new(Empty {}))
}

pub async fn RegisterDocumentHighlightProvider(
	Service:&CocoonServiceImpl,
	req:RegisterProviderRequest,
) -> Result<Response<Empty>, Status> {
	dev_log!("cocoon", "[CocoonService] Registering Document Highlight Provider");
	Service.RegisterProvider(
		req.handle,
		ProviderType::DocumentHighlight,
		&req.language_selector,
		&req.extension_id,
	);
	Ok(Response::new(Empty {}))
}

pub async fn RegisterDocumentSymbolProvider(
	Service:&CocoonServiceImpl,
	req:RegisterProviderRequest,
) -> Result<Response<Empty>, Status> {
	dev_log!("cocoon", "[CocoonService] Registering Document Symbol Provider");
	Service.RegisterProvider(
		req.handle,
		ProviderType::DocumentSymbol,
		&req.language_selector,
		&req.extension_id,
	);
	Ok(Response::new(Empty {}))
}

pub async fn RegisterWorkspaceSymbolProvider(
	Service:&CocoonServiceImpl,
	req:RegisterProviderRequest,
) -> Result<Response<Empty>, Status> {
	dev_log!("cocoon", "[CocoonService] Registering Workspace Symbol Provider");
	Service.RegisterProvider(
		req.handle,
		ProviderType::WorkspaceSymbol,
		&req.language_selector,
		&req.extension_id,
	);
	Ok(Response::new(Empty {}))
}

pub async fn RegisterRenameProvider(
	Service:&CocoonServiceImpl,
	req:RegisterProviderRequest,
) -> Result<Response<Empty>, Status> {
	dev_log!("cocoon", "[CocoonService] Registering Rename Provider");
	Service.RegisterProvider(req.handle, ProviderType::Rename, &req.language_selector, &req.extension_id);
	Ok(Response::new(Empty {}))
}

pub async fn RegisterDocumentFormattingProvider(
	Service:&CocoonServiceImpl,
	req:RegisterProviderRequest,
) -> Result<Response<Empty>, Status> {
	dev_log!("cocoon", "[CocoonService] Registering Document Formatting Provider");
	Service.RegisterProvider(
		req.handle,
		ProviderType::DocumentFormatting,
		&req.language_selector,
		&req.extension_id,
	);
	Ok(Response::new(Empty {}))
}

pub async fn RegisterDocumentRangeFormattingProvider(
	Service:&CocoonServiceImpl,
	req:RegisterProviderRequest,
) -> Result<Response<Empty>, Status> {
	dev_log!("cocoon", "[CocoonService] Registering Document Range Formatting Provider");
	Service.RegisterProvider(
		req.handle,
		ProviderType::DocumentRangeFormatting,
		&req.language_selector,
		&req.extension_id,
	);
	Ok(Response::new(Empty {}))
}

pub async fn RegisterOnTypeFormattingProvider(
	Service:&CocoonServiceImpl,
	req:RegisterOnTypeFormattingProviderRequest,
) -> Result<Response<Empty>, Status> {
	dev_log!("cocoon", "[CocoonService] Registering On Type Formatting Provider");
	Service.RegisterProvider(
		req.handle,
		ProviderType::OnTypeFormatting,
		&req.language_selector,
		&req.extension_id,
	);
	Ok(Response::new(Empty {}))
}

pub async fn RegisterSignatureHelpProvider(
	Service:&CocoonServiceImpl,
	req:RegisterSignatureHelpProviderRequest,
) -> Result<Response<Empty>, Status> {
	dev_log!("cocoon", "[CocoonService] Registering Signature Help Provider");
	Service.RegisterProvider(
		req.handle,
		ProviderType::SignatureHelp,
		&req.language_selector,
		&req.extension_id,
	);
	Ok(Response::new(Empty {}))
}

pub async fn RegisterCodeLensProvider(
	Service:&CocoonServiceImpl,
	req:RegisterProviderRequest,
) -> Result<Response<Empty>, Status> {
	dev_log!("cocoon", "[CocoonService] Registering Code Lens Provider");
	Service.RegisterProvider(req.handle, ProviderType::CodeLens, &req.language_selector, &req.extension_id);
	Ok(Response::new(Empty {}))
}

pub async fn RegisterFoldingRangeProvider(
	Service:&CocoonServiceImpl,
	req:RegisterProviderRequest,
) -> Result<Response<Empty>, Status> {
	dev_log!("cocoon", "[CocoonService] Registering Folding Range Provider");
	Service.RegisterProvider(
		req.handle,
		ProviderType::FoldingRange,
		&req.language_selector,
		&req.extension_id,
	);
	Ok(Response::new(Empty {}))
}

pub async fn RegisterSelectionRangeProvider(
	Service:&CocoonServiceImpl,
	req:RegisterProviderRequest,
) -> Result<Response<Empty>, Status> {
	dev_log!("cocoon", "[CocoonService] Registering Selection Range Provider");
	Service.RegisterProvider(
		req.handle,
		ProviderType::SelectionRange,
		&req.language_selector,
		&req.extension_id,
	);
	Ok(Response::new(Empty {}))
}

pub async fn RegisterSemanticTokensProvider(
	Service:&CocoonServiceImpl,
	req:RegisterSemanticTokensProviderRequest,
) -> Result<Response<Empty>, Status> {
	dev_log!("cocoon", "[CocoonService] Registering Semantic Tokens Provider");
	Service.RegisterProvider(
		req.handle,
		ProviderType::SemanticTokens,
		&req.language_selector,
		&req.extension_id,
	);
	Ok(Response::new(Empty {}))
}

pub async fn RegisterInlayHintsProvider(
	Service:&CocoonServiceImpl,
	req:RegisterProviderRequest,
) -> Result<Response<Empty>, Status> {
	dev_log!("cocoon", "[CocoonService] Registering Inlay Hints Provider");
	Service.RegisterProvider(req.handle, ProviderType::InlayHint, &req.language_selector, &req.extension_id);
	Ok(Response::new(Empty {}))
}

pub async fn RegisterTypeHierarchyProvider(
	Service:&CocoonServiceImpl,
	req:RegisterProviderRequest,
) -> Result<Response<Empty>, Status> {
	dev_log!("cocoon", "[CocoonService] Registering Type Hierarchy Provider");
	Service.RegisterProvider(
		req.handle,
		ProviderType::TypeHierarchy,
		&req.language_selector,
		&req.extension_id,
	);
	Ok(Response::new(Empty {}))
}

pub async fn RegisterCallHierarchyProvider(
	Service:&CocoonServiceImpl,
	req:RegisterProviderRequest,
) -> Result<Response<Empty>, Status> {
	dev_log!("cocoon", "[CocoonService] Registering Call Hierarchy Provider");
	Service.RegisterProvider(
		req.handle,
		ProviderType::CallHierarchy,
		&req.language_selector,
		&req.extension_id,
	);
	Ok(Response::new(Empty {}))
}

pub async fn RegisterLinkedEditingRangeProvider(
	Service:&CocoonServiceImpl,
	req:RegisterProviderRequest,
) -> Result<Response<Empty>, Status> {
	dev_log!("cocoon", "[CocoonService] Registering Linked Editing Range Provider");
	Service.RegisterProvider(
		req.handle,
		ProviderType::LinkedEditingRange,
		&req.language_selector,
		&req.extension_id,
	);
	Ok(Response::new(Empty {}))
}

// ==================== Provide Handlers ====================

pub async fn ProvideHover(
	Service:&CocoonServiceImpl,
	req:ProvideHoverRequest,
) -> Result<Response<ProvideHoverResponse>, Status> {
	let uri_string = req.uri.as_ref().map(|u| u.value.as_str()).unwrap_or("");
	let position = req.position.as_ref();
	let line = position.map(|p| p.line).unwrap_or(0);
	let character = position.map(|p| p.character).unwrap_or(0);
	dev_log!(
		"provider",
		"ProvideHover entry handle={} uri={} line={} char={}",
		req.provider_handle,
		uri_string,
		line,
		character
	);

	let document_uri = Url::parse(uri_string).map_err(|e| Status::invalid_argument(format!("Invalid URI: {}", e)))?;

	let position_dto = PositionDTO { LineNumber:line, Column:character };

	match Service.environment.ProvideHover(document_uri, position_dto).await {
		Ok(Some(hover)) => {
			let markdown = hover
				.Contents
				.iter()
				.map(|c| c.Value.as_str())
				.collect::<Vec<_>>()
				.join("\n---\n");
			let range = hover.Range.map(|r| {
				Range {
					start:Some(Position { line:r.StartLineNumber, character:r.StartColumn }),
					end:Some(Position { line:r.EndLineNumber, character:r.EndColumn }),
				}
			});
			dev_log!(
				"provider",
				"ProvideHover result handle={} contents_len={} hasRange={}",
				req.provider_handle,
				markdown.len(),
				range.is_some()
			);
			Ok(Response::new(ProvideHoverResponse { markdown, range }))
		},
		Ok(None) => {
			dev_log!("provider", "ProvideHover result handle={} (no provider)", req.provider_handle);
			Ok(Response::new(ProvideHoverResponse { markdown:String::new(), range:None }))
		},
		Err(e) => {
			dev_log!("provider", "warn: ProvideHover failed handle={} err={}", req.provider_handle, e);
			Err(Status::internal(format!("Hover failed: {}", e)))
		},
	}
}

pub async fn ProvideCompletionItems(
	Service:&CocoonServiceImpl,
	req:ProvideCompletionItemsRequest,
) -> Result<Response<ProvideCompletionItemsResponse>, Status> {
	dev_log!(
		"cocoon",
		"[CocoonService] Providing completions for provider {}",
		req.provider_handle
	);

	let uri_string = req.uri.as_ref().map(|u| u.value.as_str()).unwrap_or("");
	let document_uri = Url::parse(uri_string).map_err(|e| Status::invalid_argument(format!("Invalid URI: {}", e)))?;
	let position = req.position.as_ref();
	let position_dto = PositionDTO {
		LineNumber:position.map(|p| p.line).unwrap_or(0),
		Column:position.map(|p| p.character).unwrap_or(0),
	};
	let context_dto = CommonLibrary::LanguageFeature::DTO::CompletionContextDTO::CompletionContextDTO {
		TriggerKind:CommonLibrary::LanguageFeature::DTO::CompletionContextDTO::CompletionTriggerKindDTO::Invoke,
		TriggerCharacter:if req.trigger_character.is_empty() {
			None
		} else {
			Some(req.trigger_character.clone())
		},
	};

	match Service
		.environment
		.ProvideCompletions(document_uri, position_dto, context_dto, None)
		.await
	{
		Ok(Some(list)) => {
			let items = list
				.Suggestions
				.iter()
				.map(|s| {
					CompletionItem {
						label:s.Label.as_str().map(|l| l.to_string()).unwrap_or_default(),
						kind:format!("{}", s.Kind),
						detail:s.Detail.clone().unwrap_or_default(),
						documentation:Vec::new(),
						insert_text:s.InsertText.as_ref().and_then(|v| v.as_str()).unwrap_or("").to_string(),
					}
				})
				.collect();
			Ok(Response::new(ProvideCompletionItemsResponse { items }))
		},
		Ok(None) => Ok(Response::new(ProvideCompletionItemsResponse { items:Vec::new() })),
		Err(e) => Err(Status::internal(format!("Completions failed: {}", e))),
	}
}

pub async fn ProvideDefinition(
	Service:&CocoonServiceImpl,
	req:ProvideDefinitionRequest,
) -> Result<Response<ProvideDefinitionResponse>, Status> {
	dev_log!(
		"cocoon",
		"[CocoonService] Providing definition for provider {}",
		req.provider_handle
	);

	let uri_string = req.uri.as_ref().map(|u| u.value.as_str()).unwrap_or("");
	let document_uri = Url::parse(uri_string).map_err(|e| Status::invalid_argument(format!("Invalid URI: {}", e)))?;
	let position = req.position.as_ref();
	let position_dto = PositionDTO {
		LineNumber:position.map(|p| p.line).unwrap_or(0),
		Column:position.map(|p| p.character).unwrap_or(0),
	};

	match Service.environment.ProvideDefinition(document_uri, position_dto).await {
		Ok(Some(locations)) => {
			let proto_locations = locations
				.iter()
				.map(|loc| {
					Location {
						uri:Some(Uri { value:loc.Uri.to_string() }),
						range:Some(Range {
							start:Some(Position { line:loc.Range.StartLineNumber, character:loc.Range.StartColumn }),
							end:Some(Position { line:loc.Range.EndLineNumber, character:loc.Range.EndColumn }),
						}),
					}
				})
				.collect();
			Ok(Response::new(ProvideDefinitionResponse { locations:proto_locations }))
		},
		Ok(None) => Ok(Response::new(ProvideDefinitionResponse { locations:Vec::new() })),
		Err(e) => Err(Status::internal(format!("Definition failed: {}", e))),
	}
}

pub async fn ProvideReferences(
	Service:&CocoonServiceImpl,
	req:ProvideReferencesRequest,
) -> Result<Response<ProvideReferencesResponse>, Status> {
	dev_log!(
		"cocoon",
		"[CocoonService] Providing references for provider {}",
		req.provider_handle
	);

	let uri_string = req.uri.as_ref().map(|u| u.value.as_str()).unwrap_or("");
	let document_uri = Url::parse(uri_string).map_err(|e| Status::invalid_argument(format!("Invalid URI: {}", e)))?;
	let position = req.position.as_ref();
	let position_dto = PositionDTO {
		LineNumber:position.map(|p| p.line).unwrap_or(0),
		Column:position.map(|p| p.character).unwrap_or(0),
	};
	let context_dto = json!({ "includeDeclaration": true });

	match Service
		.environment
		.ProvideReferences(document_uri, position_dto, context_dto)
		.await
	{
		Ok(Some(locations)) => {
			let proto_locations = locations
				.iter()
				.map(|loc| {
					Location {
						uri:Some(Uri { value:loc.Uri.to_string() }),
						range:Some(Range {
							start:Some(Position { line:loc.Range.StartLineNumber, character:loc.Range.StartColumn }),
							end:Some(Position { line:loc.Range.EndLineNumber, character:loc.Range.EndColumn }),
						}),
					}
				})
				.collect();
			Ok(Response::new(ProvideReferencesResponse { locations:proto_locations }))
		},
		Ok(None) => Ok(Response::new(ProvideReferencesResponse { locations:Vec::new() })),
		Err(e) => Err(Status::internal(format!("References failed: {}", e))),
	}
}

pub async fn ProvideCodeActions(
	Service:&CocoonServiceImpl,
	req:ProvideCodeActionsRequest,
) -> Result<Response<ProvideCodeActionsResponse>, Status> {
	dev_log!(
		"cocoon",
		"[CocoonService] Providing code actions for provider {}",
		req.provider_handle
	);

	let uri_string = req.uri.as_ref().map(|u| u.value.as_str()).unwrap_or("");
	let document_uri = Url::parse(uri_string).map_err(|e| Status::invalid_argument(format!("Invalid URI: {}", e)))?;
	let range = req.range.as_ref();
	let range_dto = json!({
		"StartLineNumber": range.and_then(|r| r.start.as_ref()).map(|p| p.line).unwrap_or(0),
		"StartColumn": range.and_then(|r| r.start.as_ref()).map(|p| p.character).unwrap_or(0),
		"EndLineNumber": range.and_then(|r| r.end.as_ref()).map(|p| p.line).unwrap_or(0),
		"EndColumn": range.and_then(|r| r.end.as_ref()).map(|p| p.character).unwrap_or(0),
	});
	let context_dto = json!({ "diagnostics": [], "only": null });

	match Service
		.environment
		.ProvideCodeActions(document_uri, range_dto, context_dto)
		.await
	{
		Ok(Some(_value)) => Ok(Response::new(ProvideCodeActionsResponse { actions:Vec::new() })),
		Ok(None) => Ok(Response::new(ProvideCodeActionsResponse { actions:Vec::new() })),
		Err(e) => Err(Status::internal(format!("Code actions failed: {}", e))),
	}
}

pub async fn ProvideDocumentHighlights(
	Service:&CocoonServiceImpl,
	req:ProvideDocumentHighlightsRequest,
) -> Result<Response<ProvideDocumentHighlightsResponse>, Status> {
	dev_log!("cocoon", "[CocoonService] Providing document highlights");

	let uri_string = req.uri.as_ref().map(|u| u.value.as_str()).unwrap_or("");
	let document_uri = Url::parse(uri_string).map_err(|e| Status::invalid_argument(format!("Invalid URI: {}", e)))?;
	let position = req.position.as_ref();
	let position_dto = PositionDTO {
		LineNumber:position.map(|p| p.line).unwrap_or(0),
		Column:position.map(|p| p.character).unwrap_or(0),
	};

	match Service.environment.ProvideDocumentHighlights(document_uri, position_dto).await {
		Ok(_result) => Ok(Response::new(ProvideDocumentHighlightsResponse::default())),
		Err(e) => Err(Status::internal(format!("Document highlights failed: {}", e))),
	}
}

pub async fn ProvideDocumentSymbols(
	Service:&CocoonServiceImpl,
	req:ProvideDocumentSymbolsRequest,
) -> Result<Response<ProvideDocumentSymbolsResponse>, Status> {
	dev_log!("cocoon", "[CocoonService] Providing document symbols");

	let uri_string = req.uri.as_ref().map(|u| u.value.as_str()).unwrap_or("");
	let document_uri = Url::parse(uri_string).map_err(|e| Status::invalid_argument(format!("Invalid URI: {}", e)))?;

	match Service.environment.ProvideDocumentSymbols(document_uri).await {
		Ok(_result) => Ok(Response::new(ProvideDocumentSymbolsResponse::default())),
		Err(e) => Err(Status::internal(format!("Document symbols failed: {}", e))),
	}
}

pub async fn ProvideWorkspaceSymbols(
	Service:&CocoonServiceImpl,
	req:ProvideWorkspaceSymbolsRequest,
) -> Result<Response<ProvideWorkspaceSymbolsResponse>, Status> {
	dev_log!("cocoon", "[CocoonService] Providing workspace symbols for query: {}", req.query);

	match Service.environment.ProvideWorkspaceSymbols(req.query).await {
		Ok(_result) => Ok(Response::new(ProvideWorkspaceSymbolsResponse::default())),
		Err(e) => Err(Status::internal(format!("Workspace symbols failed: {}", e))),
	}
}

pub async fn ProvideRenameEdits(
	Service:&CocoonServiceImpl,
	req:ProvideRenameEditsRequest,
) -> Result<Response<ProvideRenameEditsResponse>, Status> {
	dev_log!("cocoon", "[CocoonService] Providing rename edits: new_name={}", req.new_name);

	let uri_string = req.uri.as_ref().map(|u| u.value.as_str()).unwrap_or("");
	let document_uri = Url::parse(uri_string).map_err(|e| Status::invalid_argument(format!("Invalid URI: {}", e)))?;
	let position = req.position.as_ref();
	let position_dto = PositionDTO {
		LineNumber:position.map(|p| p.line).unwrap_or(0),
		Column:position.map(|p| p.character).unwrap_or(0),
	};

	match Service
		.environment
		.ProvideRenameEdits(document_uri, position_dto, req.new_name)
		.await
	{
		Ok(_result) => Ok(Response::new(ProvideRenameEditsResponse::default())),
		Err(e) => Err(Status::internal(format!("Rename edits failed: {}", e))),
	}
}

pub async fn ProvideDocumentFormatting(
	Service:&CocoonServiceImpl,
	req:ProvideDocumentFormattingRequest,
) -> Result<Response<ProvideDocumentFormattingResponse>, Status> {
	dev_log!("cocoon", "[CocoonService] Providing document formatting");

	let uri_string = req.uri.as_ref().map(|u| u.value.as_str()).unwrap_or("");
	let document_uri = Url::parse(uri_string).map_err(|e| Status::invalid_argument(format!("Invalid URI: {}", e)))?;
	let options_dto = json!({ "tabSize": 4, "insertSpaces": true });

	match Service
		.environment
		.ProvideDocumentFormattingEdits(document_uri, options_dto)
		.await
	{
		Ok(_result) => Ok(Response::new(ProvideDocumentFormattingResponse::default())),
		Err(e) => Err(Status::internal(format!("Document formatting failed: {}", e))),
	}
}

pub async fn ProvideDocumentRangeFormatting(
	Service:&CocoonServiceImpl,
	req:ProvideDocumentRangeFormattingRequest,
) -> Result<Response<ProvideDocumentRangeFormattingResponse>, Status> {
	dev_log!("cocoon", "[CocoonService] Providing document range formatting");

	let uri_string = req.uri.as_ref().map(|u| u.value.as_str()).unwrap_or("");
	let document_uri = Url::parse(uri_string).map_err(|e| Status::invalid_argument(format!("Invalid URI: {}", e)))?;
	let range = req.range.as_ref();
	let range_dto = json!({
		"StartLineNumber": range.and_then(|r| r.start.as_ref()).map(|p| p.line).unwrap_or(0),
		"StartColumn": range.and_then(|r| r.start.as_ref()).map(|p| p.character).unwrap_or(0),
		"EndLineNumber": range.and_then(|r| r.end.as_ref()).map(|p| p.line).unwrap_or(0),
		"EndColumn": range.and_then(|r| r.end.as_ref()).map(|p| p.character).unwrap_or(0),
	});
	let options_dto = json!({ "tabSize": 4, "insertSpaces": true });

	match Service
		.environment
		.ProvideDocumentRangeFormattingEdits(document_uri, range_dto, options_dto)
		.await
	{
		Ok(_result) => Ok(Response::new(ProvideDocumentRangeFormattingResponse::default())),
		Err(e) => Err(Status::internal(format!("Document range formatting failed: {}", e))),
	}
}

pub async fn ProvideOnTypeFormatting(
	Service:&CocoonServiceImpl,
	req:ProvideOnTypeFormattingRequest,
) -> Result<Response<ProvideOnTypeFormattingResponse>, Status> {
	dev_log!("cocoon", "[CocoonService] Providing on-type formatting");

	let uri_string = req.uri.as_ref().map(|u| u.value.as_str()).unwrap_or("");
	let document_uri = Url::parse(uri_string).map_err(|e| Status::invalid_argument(format!("Invalid URI: {}", e)))?;
	let position = req.position.as_ref();
	let position_dto = PositionDTO {
		LineNumber:position.map(|p| p.line).unwrap_or(0),
		Column:position.map(|p| p.character).unwrap_or(0),
	};
	let options_dto = json!({ "tabSize": 4, "insertSpaces": true });

	match Service
		.environment
		.ProvideOnTypeFormattingEdits(document_uri, position_dto, req.character, options_dto)
		.await
	{
		Ok(_result) => Ok(Response::new(ProvideOnTypeFormattingResponse::default())),
		Err(e) => Err(Status::internal(format!("On-type formatting failed: {}", e))),
	}
}

pub async fn ProvideSignatureHelp(
	Service:&CocoonServiceImpl,
	req:ProvideSignatureHelpRequest,
) -> Result<Response<ProvideSignatureHelpResponse>, Status> {
	dev_log!("cocoon", "[CocoonService] Providing signature help");

	let uri_string = req.uri.as_ref().map(|u| u.value.as_str()).unwrap_or("");
	let document_uri = Url::parse(uri_string).map_err(|e| Status::invalid_argument(format!("Invalid URI: {}", e)))?;
	let position = req.position.as_ref();
	let position_dto = PositionDTO {
		LineNumber:position.map(|p| p.line).unwrap_or(0),
		Column:position.map(|p| p.character).unwrap_or(0),
	};
	let context_dto = json!({ "triggerKind": 1, "isRetrigger": false });

	match Service
		.environment
		.ProvideSignatureHelp(document_uri, position_dto, context_dto)
		.await
	{
		Ok(_result) => Ok(Response::new(ProvideSignatureHelpResponse::default())),
		Err(e) => Err(Status::internal(format!("Signature help failed: {}", e))),
	}
}

pub async fn ProvideCodeLenses(
	Service:&CocoonServiceImpl,
	req:ProvideCodeLensesRequest,
) -> Result<Response<ProvideCodeLensesResponse>, Status> {
	dev_log!("cocoon", "[CocoonService] Providing code lenses");

	let uri_string = req.uri.as_ref().map(|u| u.value.as_str()).unwrap_or("");
	let document_uri = Url::parse(uri_string).map_err(|e| Status::invalid_argument(format!("Invalid URI: {}", e)))?;

	match Service.environment.ProvideCodeLenses(document_uri).await {
		Ok(_result) => Ok(Response::new(ProvideCodeLensesResponse::default())),
		Err(e) => Err(Status::internal(format!("Code lenses failed: {}", e))),
	}
}

pub async fn ProvideFoldingRanges(
	Service:&CocoonServiceImpl,
	req:ProvideFoldingRangesRequest,
) -> Result<Response<ProvideFoldingRangesResponse>, Status> {
	dev_log!("cocoon", "[CocoonService] Providing folding ranges");

	let uri_string = req.uri.as_ref().map(|u| u.value.as_str()).unwrap_or("");
	let document_uri = Url::parse(uri_string).map_err(|e| Status::invalid_argument(format!("Invalid URI: {}", e)))?;

	match Service.environment.ProvideFoldingRanges(document_uri).await {
		Ok(_result) => Ok(Response::new(ProvideFoldingRangesResponse::default())),
		Err(e) => Err(Status::internal(format!("Folding ranges failed: {}", e))),
	}
}

pub async fn ProvideSelectionRanges(
	Service:&CocoonServiceImpl,
	req:ProvideSelectionRangesRequest,
) -> Result<Response<ProvideSelectionRangesResponse>, Status> {
	dev_log!("cocoon", "[CocoonService] Providing selection ranges");

	let uri_string = req.uri.as_ref().map(|u| u.value.as_str()).unwrap_or("");
	let document_uri = Url::parse(uri_string).map_err(|e| Status::invalid_argument(format!("Invalid URI: {}", e)))?;
	let PositionDTOs:Vec<PositionDTO> = req
		.positions
		.iter()
		.map(|P| PositionDTO { LineNumber:P.line, Column:P.character })
		.collect();

	match Service.environment.ProvideSelectionRanges(document_uri, PositionDTOs).await {
		Ok(_result) => Ok(Response::new(ProvideSelectionRangesResponse::default())),
		Err(e) => Err(Status::internal(format!("Selection ranges failed: {}", e))),
	}
}

pub async fn ProvideSemanticTokensFull(
	Service:&CocoonServiceImpl,
	req:ProvideSemanticTokensRequest,
) -> Result<Response<ProvideSemanticTokensResponse>, Status> {
	dev_log!("cocoon", "[CocoonService] Providing semantic tokens");

	let uri_string = req.uri.as_ref().map(|u| u.value.as_str()).unwrap_or("");
	let document_uri = Url::parse(uri_string).map_err(|e| Status::invalid_argument(format!("Invalid URI: {}", e)))?;

	match Service.environment.ProvideSemanticTokensFull(document_uri).await {
		Ok(_result) => Ok(Response::new(ProvideSemanticTokensResponse::default())),
		Err(e) => Err(Status::internal(format!("Semantic tokens failed: {}", e))),
	}
}

pub async fn ProvideInlayHints(
	Service:&CocoonServiceImpl,
	req:ProvideInlayHintsRequest,
) -> Result<Response<ProvideInlayHintsResponse>, Status> {
	dev_log!("cocoon", "[CocoonService] Providing inlay hints");

	let uri_string = req.uri.as_ref().map(|u| u.value.as_str()).unwrap_or("");
	let document_uri = Url::parse(uri_string).map_err(|e| Status::invalid_argument(format!("Invalid URI: {}", e)))?;
	let range = req.range.as_ref();
	let range_dto = json!({
		"StartLineNumber": range.and_then(|r| r.start.as_ref()).map(|p| p.line).unwrap_or(0),
		"StartColumn": range.and_then(|r| r.start.as_ref()).map(|p| p.character).unwrap_or(0),
		"EndLineNumber": range.and_then(|r| r.end.as_ref()).map(|p| p.line).unwrap_or(0),
		"EndColumn": range.and_then(|r| r.end.as_ref()).map(|p| p.character).unwrap_or(0),
	});

	match Service.environment.ProvideInlayHints(document_uri, range_dto).await {
		Ok(_result) => Ok(Response::new(ProvideInlayHintsResponse::default())),
		Err(e) => Err(Status::internal(format!("Inlay hints failed: {}", e))),
	}
}

pub async fn ProvideTypeHierarchySupertypes(
	Service:&CocoonServiceImpl,
	req:ProvideTypeHierarchyRequest,
) -> Result<Response<ProvideTypeHierarchyResponse>, Status> {
	dev_log!("cocoon", "[CocoonService] Providing type hierarchy supertypes");

	let item_dto = json!({
		"name": req.item.as_ref().map(|i| i.name.as_str()).unwrap_or(""),
		"uri": req.uri.as_ref().map(|u| u.value.as_str()).unwrap_or(""),
	});
	match Service.environment.ProvideTypeHierarchySupertypes(item_dto).await {
		Ok(_result) => Ok(Response::new(ProvideTypeHierarchyResponse::default())),
		Err(e) => Err(Status::internal(format!("Type hierarchy supertypes failed: {}", e))),
	}
}

pub async fn ProvideTypeHierarchySubtypes(
	Service:&CocoonServiceImpl,
	req:ProvideTypeHierarchyRequest,
) -> Result<Response<ProvideTypeHierarchyResponse>, Status> {
	dev_log!("cocoon", "[CocoonService] Providing type hierarchy subtypes");

	let item_dto = json!({
		"name": req.item.as_ref().map(|i| i.name.as_str()).unwrap_or(""),
		"uri": req.uri.as_ref().map(|u| u.value.as_str()).unwrap_or(""),
	});
	match Service.environment.ProvideTypeHierarchySubtypes(item_dto).await {
		Ok(_result) => Ok(Response::new(ProvideTypeHierarchyResponse::default())),
		Err(e) => Err(Status::internal(format!("Type hierarchy subtypes failed: {}", e))),
	}
}

pub async fn ProvideCallHierarchyIncomingCalls(
	Service:&CocoonServiceImpl,
	req:ProvideCallHierarchyRequest,
) -> Result<Response<ProvideCallHierarchyResponse>, Status> {
	dev_log!("cocoon", "[CocoonService] Providing call hierarchy incoming");

	let item_dto = json!({
		"name": req.item.as_ref().map(|i| i.name.as_str()).unwrap_or(""),
		"uri": req.uri.as_ref().map(|u| u.value.as_str()).unwrap_or(""),
	});
	match Service.environment.ProvideCallHierarchyIncomingCalls(item_dto).await {
		Ok(_result) => Ok(Response::new(ProvideCallHierarchyResponse::default())),
		Err(e) => Err(Status::internal(format!("Call hierarchy incoming failed: {}", e))),
	}
}

pub async fn ProvideCallHierarchyOutgoingCalls(
	Service:&CocoonServiceImpl,
	req:ProvideCallHierarchyRequest,
) -> Result<Response<ProvideCallHierarchyResponse>, Status> {
	dev_log!("cocoon", "[CocoonService] Providing call hierarchy outgoing");

	let item_dto = json!({
		"name": req.item.as_ref().map(|i| i.name.as_str()).unwrap_or(""),
		"uri": req.uri.as_ref().map(|u| u.value.as_str()).unwrap_or(""),
	});
	match Service.environment.ProvideCallHierarchyOutgoingCalls(item_dto).await {
		Ok(_result) => Ok(Response::new(ProvideCallHierarchyResponse::default())),
		Err(e) => Err(Status::internal(format!("Call hierarchy outgoing failed: {}", e))),
	}
}

pub async fn ProvideLinkedEditingRanges(
	Service:&CocoonServiceImpl,
	req:ProvideLinkedEditingRangesRequest,
) -> Result<Response<ProvideLinkedEditingRangesResponse>, Status> {
	dev_log!("cocoon", "[CocoonService] Providing linked editing ranges");

	let uri_string = req.uri.as_ref().map(|u| u.value.as_str()).unwrap_or("");
	let document_uri = Url::parse(uri_string).map_err(|e| Status::invalid_argument(format!("Invalid URI: {}", e)))?;
	let position = req.position.as_ref();
	let position_dto = PositionDTO {
		LineNumber:position.map(|p| p.line).unwrap_or(0),
		Column:position.map(|p| p.character).unwrap_or(0),
	};

	match Service.environment.ProvideLinkedEditingRanges(document_uri, position_dto).await {
		Ok(_result) => Ok(Response::new(ProvideLinkedEditingRangesResponse::default())),
		Err(e) => Err(Status::internal(format!("Linked editing ranges failed: {}", e))),
	}
}
