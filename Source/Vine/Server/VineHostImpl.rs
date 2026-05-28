//! Mountain's implementation of [`::Vine::Host::VineHost`] for
//! [`MountainVinegRPCService`]. Lets the canonical handler tree in the
//! Vine crate operate against `&dyn VineHost` while reusing Mountain's
//! `AppHandle`-based `emit` plumbing and IPC bus.

use std::sync::Arc;

use serde_json::Value;
use tauri::{AppHandle, Emitter};
use ::Vine::Host::{ApplicationStateAccess, IPCProvider, RendererEmitter, VineHost};

use crate::{Vine::Server::MountainVinegRPCService::MountainVinegRPCService, dev_log};

/// Minimal `ApplicationStateAccess` carrier for the Mountain embedder.
/// Vine handlers only need the embedder label today; richer state lives
/// behind Mountain-local sub-traits added as port families need them.
struct MountainApplicationStateAccess;

impl ApplicationStateAccess for MountainApplicationStateAccess {
	fn EmbedderName(&self) -> &'static str { "Mountain" }
}

static MOUNTAIN_APP_STATE:MountainApplicationStateAccess = MountainApplicationStateAccess;

/// Cheap-to-clone renderer event sink. Internally holds a
/// [`tauri::AppHandle`], which is itself a thin `Arc` wrapper - cloning
/// is a ref-count bump. Used by Vine handlers with long-lived flushers
/// (`ProgressReport`, `DecorationTypeLifecycle`, `OutputChannelCoalesce`,
/// `SetTextEditorDecorations`, `RegisterCommand`) that emit from a
/// background task.
pub struct TauriRendererEmitter {
	Handle:AppHandle,
}

impl TauriRendererEmitter {
	pub fn New(Handle:AppHandle) -> Self { Self { Handle } }
}

impl RendererEmitter for TauriRendererEmitter {
	fn Emit(&self, Channel:&str, Payload:Value) {
		if let Err(Error) = self.Handle.emit(Channel, Payload) {
			dev_log!("sky-emit", "[SkyEmit] fail channel={} error={}", Channel, Error);
		}
	}
}

/// IPC bridge that routes `SendNotification` calls to the Vine gRPC client
/// so breakpoint fan-backs and similar cross-extension notifications reach
/// Cocoon. `SendRequest` is left as a no-op until a handler needs it.
struct MountainIPCProvider;

impl IPCProvider for MountainIPCProvider {
	fn SendRequest(
		&self,
		Channel:&str,
		_Payload:Value,
	) -> futures::future::BoxFuture<'_, ::Vine::Error::Result<Value>> {
		let Channel = Channel.to_string();

		Box::pin(async move {
			dev_log!(
				"grpc",
				"warn: [VineHost] IPCProvider::SendRequest channel={} - not wired",
				Channel
			);

			Ok(Value::Null)
		})
	}

	fn SendNotification(&self, Channel:&str, Method:&str, Payload:Value) {
		let Ch = Channel.to_string();
		let M = Method.to_string();

		tauri::async_runtime::spawn(async move {
			let _ = crate::Vine::Client::SendNotification::Fn(Ch, M, Payload).await;
		});
	}
}

impl VineHost for MountainVinegRPCService {
	fn ApplicationState(&self) -> &dyn ApplicationStateAccess { &MOUNTAIN_APP_STATE }

	fn EmitToRenderer(&self, Channel:&str, Payload:Value) {
		if let Err(Error) = self.ApplicationHandle().emit(Channel, Payload) {
			dev_log!("sky-emit", "[SkyEmit] fail channel={} error={}", Channel, Error);
		}
	}

	fn RendererEmitter(&self) -> Arc<dyn RendererEmitter> {
		Arc::new(TauriRendererEmitter::New(self.ApplicationHandle().clone()))
	}

	fn IPCProvider(&self) -> Arc<dyn IPCProvider> { Arc::new(MountainIPCProvider) }

	fn UnregisterProvider(&self, Handle:u32) {
		self.RunTime()
			.Environment
			.ApplicationState
			.Extension
			.ProviderRegistration
			.UnregisterProvider(Handle);
	}

	fn RegisterCommandInRegistry(&self, CommandId:&str, SideCarIdentifier:&str) {
		use crate::Environment::CommandProvider::CommandHandler;

		if let Ok(mut Registry) = self
			.RunTime()
			.Environment
			.ApplicationState
			.Extension
			.Registry
			.CommandRegistry
			.lock()
		{
			Registry.insert(
				CommandId.to_string(),
				CommandHandler::Proxied {
					SideCarIdentifier:SideCarIdentifier.to_string(),
					CommandIdentifier:CommandId.to_string(),
				},
			);
		}
	}

	fn UnregisterCommandInRegistry(&self, CommandId:&str) {
		if let Ok(mut Registry) = self
			.RunTime()
			.Environment
			.ApplicationState
			.Extension
			.Registry
			.CommandRegistry
			.lock()
		{
			Registry.remove(CommandId);
		}
	}

	fn SpawnSendTextToTerminal(&self, TerminalId:u64, Text:String) {
		use CommonLibrary::{Environment::Requires::Requires, Terminal::TerminalProvider::TerminalProvider};

		let Provider:Arc<dyn TerminalProvider> = self.RunTime().Environment.Require();

		tauri::async_runtime::spawn(async move {
			let _ = Provider.SendTextToTerminal(TerminalId, Text).await;
		});
	}

	fn SpawnDisposeTerminal(&self, TerminalId:u64) {
		use CommonLibrary::{Environment::Requires::Requires, Terminal::TerminalProvider::TerminalProvider};

		let Provider:Arc<dyn TerminalProvider> = self.RunTime().Environment.Require();

		tauri::async_runtime::spawn(async move {
			let _ = Provider.DisposeTerminal(TerminalId).await;
		});
	}

	fn CreateTerminal<'a>(&'a self, Options:&'a Value) -> futures::future::BoxFuture<'a, Option<Value>> {
		use CommonLibrary::{Environment::Requires::Requires, Terminal::TerminalProvider::TerminalProvider};

		let Provider:Arc<dyn TerminalProvider> = self.RunTime().Environment.Require();
		let Opts = Options.clone();

		Box::pin(async move { Provider.CreateTerminal(Opts).await.ok() })
	}

	fn RegisterScmInRegistry(&self, Handle:u32, ScmId:&str, Label:&str, ExtId:&str) {
		use CommonLibrary::LanguageFeature::DTO::ProviderType::ProviderType;
		use serde_json::json;

		use crate::ApplicationState::DTO::ProviderRegistrationDTO::ProviderRegistrationDTO;

		let Dto = ProviderRegistrationDTO {
			Handle,
			ProviderType:ProviderType::SourceControl,
			Selector:json!([{ "scmId": ScmId }]),
			SideCarIdentifier:"cocoon-main".to_string(),
			ExtensionIdentifier:json!(ExtId),
			Options:Some(json!({ "scmId": ScmId, "label": Label })),
		};

		self.RunTime()
			.Environment
			.ApplicationState
			.Extension
			.ProviderRegistration
			.RegisterProvider(Handle, Dto);
	}

	fn CreateSourceControl<'a>(&'a self, Payload:Value) -> futures::future::BoxFuture<'a, ()> {
		use CommonLibrary::SourceControlManagement::SourceControlManagementProvider::SourceControlManagementProvider;

		let RunTime = self.RunTime().clone();

		Box::pin(async move {
			if let Err(E) = RunTime.Environment.CreateSourceControl(Payload).await {
				dev_log!("grpc", "warn: [VineHost] CreateSourceControl failed: {}", E);
			}
		})
	}

	fn UpdateSourceControlGroup<'a>(&'a self, ScmHandle:u32, Payload:Value) -> futures::future::BoxFuture<'a, ()> {
		use CommonLibrary::SourceControlManagement::SourceControlManagementProvider::SourceControlManagementProvider;

		let RunTime = self.RunTime().clone();

		Box::pin(async move {
			if let Err(E) = RunTime.Environment.UpdateSourceControlGroup(ScmHandle, Payload).await {
				dev_log!(
					"grpc",
					"warn: [VineHost] UpdateSourceControlGroup scm={} failed: {}",
					ScmHandle,
					E
				);
			}
		})
	}

	fn RegisterLanguageProvider(&self, Handle:u32, TypeName:&str, Payload:&Value) -> bool {
		use CommonLibrary::LanguageFeature::DTO::ProviderType::ProviderType as PT;
		use serde_json::json;

		use crate::ApplicationState::DTO::ProviderRegistrationDTO::ProviderRegistrationDTO;

		let ProvType:Option<PT> = match TypeName {
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

		let Selector = Payload
			.get("languageSelector")
			.or_else(|| Payload.get("language_selector"))
			.and_then(Value::as_str)
			.unwrap_or("*");

		let ExtId = Payload
			.get("extensionId")
			.or_else(|| Payload.get("extension_id"))
			.and_then(Value::as_str)
			.unwrap_or("");

		let Scheme = Payload.get("scheme").and_then(Value::as_str).unwrap_or("");

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
			Options:Payload.get("options").cloned(),
		};

		self.RunTime()
			.Environment
			.ApplicationState
			.Extension
			.ProviderRegistration
			.RegisterProvider(Handle, Dto);

		true
	}
}
