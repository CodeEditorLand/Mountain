#![allow(non_snake_case)]
//! Cocoon → Mountain `register_scm_provider` notification.
//!
//! Replaces the previous behaviour where this wire-method fell through
//! the language-providers OR-block in `MountainVinegRPCService.rs` and
//! got registered as a `ProviderType::SourceControl` *language* provider
//! (wrong - the SCM viewlet binds to `ApplicationState::SourceControl`,
//! not the language-feature provider registry, so the panel stayed
//! empty even though `vscode.scm.createSourceControl(...)` succeeded
//! inside Cocoon).
//!
//! Cocoon emits this from `ScmNamespace.ts:14` with payload shape:
//!
//! ```ignore
//! { handle: u32, id, label, root_uri, extension_id }
//! ```
//!
//! Three side effects happen here:
//!   1. `ProviderRegistration::RegisterProvider` records the handle so future
//!      language-feature dispatches that look up by SCM handle (rare but
//!      possible) resolve.
//!   2. `SourceControlManagementProvider::CreateSourceControl` mutates
//!      `ApplicationState::Feature::Markers::SourceControlManagementProviders`
//!      and emits `SkyEvent::SCMProviderAdded` - this is the canonical
//!      state-tracking path the SCM view uses.
//!   3. A direct `sky://scm/register` Tauri emit covers any renderer path that
//!      listens for the simpler legacy event shape (gitlens, future custom SCM
//!      views).
//!
//! All three are best-effort and independent: the trait call may fail
//! when `root_uri` is missing (extensions occasionally register an SCM
//! before opening a folder); the registry write is infallible; the
//! Sky emit is fire-and-forget.

use serde_json::{Value, json};
use tauri::Emitter;
use CommonLibrary::SourceControlManagement::SourceControlManagementProvider::SourceControlManagementProvider;

use crate::{
	ApplicationState::DTO::ProviderRegistrationDTO::ProviderRegistrationDTO,
	Vine::Server::MountainVinegRPCService::MountainVinegRPCService,
	dev_log,
};

pub async fn RegisterScmProvider(Service:&MountainVinegRPCService, Parameter:&Value) {
	let ScmId = Parameter
		.get("scm_id")
		.or_else(|| Parameter.get("id"))
		.or_else(|| Parameter.get("scmId"))
		.and_then(Value::as_str)
		.unwrap_or("")
		.to_string();
	let Label = Parameter.get("label").and_then(Value::as_str).unwrap_or(&ScmId).to_string();
	let ExtensionId = Parameter
		.get("extension_id")
		.or_else(|| Parameter.get("extensionId"))
		.and_then(Value::as_str)
		.unwrap_or("")
		.to_string();
	let RootUri = Parameter
		.get("root_uri")
		.or_else(|| Parameter.get("rootUri"))
		.cloned()
		.unwrap_or(Value::Null);

	if ScmId.is_empty() {
		dev_log!("provider-register", "[ProviderRegister] scm skip: missing scm_id");
		return;
	}

	// Cocoon's `ScmNamespace.ts` uses a process-local sequential
	// `NextProviderHandle()` and includes that handle on the wire
	// payload. Subsequent `register_scm_resource_group`,
	// `update_scm_group`, and `unregister_scm_provider` notifications
	// reference the SAME sequential handle as `scm_handle`, so we must
	// preserve it here verbatim - otherwise the registry write below
	// keys under DJB-hash-of-id and the resource-group/update path
	// keys under Cocoon's sequential, and the SCM viewlet sees a
	// provider with no groups regardless of how many resources arrive.
	//
	// Fall back to the DJB hash only when Cocoon (or a third-party
	// caller) omits the field, so this keeps working with the legacy
	// shape without forcing a Cocoon upgrade.
	let Handle = Parameter
		.get("handle")
		.or_else(|| Parameter.get("scm_handle"))
		.or_else(|| Parameter.get("scmHandle"))
		.and_then(Value::as_u64)
		.map(|H| H as u32)
		.unwrap_or_else(|| {
			ScmId
				.as_bytes()
				.iter()
				.fold(0u32, |Acc, B| Acc.wrapping_mul(31).wrapping_add(*B as u32))
		});

	use CommonLibrary::LanguageFeature::DTO::ProviderType::ProviderType;
	let RegistrationDto = ProviderRegistrationDTO {
		Handle,
		ProviderType:ProviderType::SourceControl,
		Selector:json!([{ "scmId": &ScmId }]),
		SideCarIdentifier:"cocoon-main".to_string(),
		ExtensionIdentifier:json!(&ExtensionId),
		Options:Some(json!({ "scmId": &ScmId, "label": &Label })),
	};
	Service
		.RunTime()
		.Environment
		.ApplicationState
		.Extension
		.ProviderRegistration
		.RegisterProvider(Handle, RegistrationDto);

	// Trait wiring populates `ApplicationState::Feature::Markers`
	// + emits the typed `SkyEvent::SCMProviderAdded`. RootUri is
	// expected to be a parseable URL string; when extensions pass null
	// (rare - usually a workspace folder URI) we substitute the empty
	// `file:///` so the trait still records the provider.
	let RootUriString = match &RootUri {
		Value::String(S) => S.clone(),
		Value::Object(O) => {
			O.get("external")
				.or_else(|| O.get("path"))
				.and_then(Value::as_str)
				.map(str::to_string)
				.unwrap_or_else(|| "file:///".to_string())
		},
		_ => "file:///".to_string(),
	};
	// Field names must match `SourceControlCreateDTO`'s camelCase wire
	// shape (post-DTO-audit): `id`, `label`, `rootUri`. Earlier revisions
	// passed PascalCase keys here and the trait silently failed with
	// `missing field "id"` because the DTO's serde rename uses camelCase.
	let CreateData = json!({
		"id": &ScmId,
		"label": &Label,
		"rootUri": RootUriString,
	});
	if let Err(Error) = Service.RunTime().Environment.CreateSourceControl(CreateData).await {
		dev_log!("grpc", "warn: [Scm] CreateSourceControl trait failed for {}: {}", ScmId, Error);
	}

	// Legacy listener channel kept active alongside the typed event so
	// renderer code that hasn't migrated to the markers-backed view
	// (gitlens-side custom panels, hand-rolled tests) still sees the
	// register signal.
	let _ = Service.ApplicationHandle().emit(
		"sky://scm/register",
		json!({
			"scmId": &ScmId,
			"label": &Label,
			"rootUri": &RootUriString,
			"extensionId": &ExtensionId,
			"handle": Handle,
		}),
	);

	dev_log!(
		"grpc",
		"[Scm] register provider scmId={} label={} ext={} handle={}",
		ScmId,
		Label,
		ExtensionId,
		Handle
	);
}
