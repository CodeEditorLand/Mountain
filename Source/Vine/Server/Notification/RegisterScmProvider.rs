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
// `tauri::Emitter` previously imported for direct `.emit()` calls;
// emits now route through `LogSkyEmit` which carries the trait. No
// remaining `.emit()` callsites in this file.
use CommonLibrary::SourceControlManagement::SourceControlManagementProvider::SourceControlManagementProvider;

use crate::{
	ApplicationState::DTO::ProviderRegistrationDTO::ProviderRegistrationDTO,
	Vine::Server::MountainVinegRPCService::MountainVinegRPCService,
	dev_log,
};

pub async fn RegisterScmProvider(Service:&MountainVinegRPCService, Parameter:&Value) {
	// Wire-shape contract: producer (`Cocoon/.../ScmNamespace.ts`) emits
	// camelCase keys (`rootUri`, `extensionId`) post 2026-04-27 wire audit.
	// Probe camelCase first; keep snake_case as a transitional fallback so
	// a partial rebuild (Mountain ahead of Cocoon) doesn't silently drop.
	let ScmId = Parameter
		.get("id")
		.or_else(|| Parameter.get("scmId"))
		.or_else(|| Parameter.get("scm_id"))
		.and_then(Value::as_str)
		.unwrap_or("")
		.to_string();

	let Label = Parameter.get("label").and_then(Value::as_str).unwrap_or(&ScmId).to_string();

	let ExtensionId = Parameter
		.get("extensionId")
		.or_else(|| Parameter.get("extension_id"))
		.and_then(Value::as_str)
		.unwrap_or("")
		.to_string();

	let RootUri = Parameter
		.get("rootUri")
		.or_else(|| Parameter.get("root_uri"))
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
		.or_else(|| Parameter.get("scmHandle"))
		.or_else(|| Parameter.get("scm_handle"))
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
	//
	// vscode.git's `repository.ts:983` calls `Uri.file(repository.root)`
	// which serialises to a UriComponents object: `{scheme:"file",
	// authority:"", path:"/Volumes/...", query:"", fragment:""}`. The
	// previous extractor read `O.get("path")` which is the **path
	// component only** (no scheme prefix) and passed it through to
	// `URLSerializationHelper`'s `Url::parse(...)`, which fails with
	// "relative URL without a base" because `/Volumes/...` has no
	// scheme. Rebuild a proper `<scheme>://<authority><path>` triple
	// from the components first; only fall back to `external` (already
	// a string URL) or `path` if the triple can't be assembled.
	let BuildUrlFromComponents = |O:&serde_json::Map<String, Value>| -> Option<String> {
		let Scheme = O.get("scheme").and_then(Value::as_str)?;

		if Scheme.is_empty() {
			return None;
		}

		let Authority = O.get("authority").and_then(Value::as_str).unwrap_or("");

		let Path = O.get("path").and_then(Value::as_str).unwrap_or("");

		let Query = O.get("query").and_then(Value::as_str).unwrap_or("");

		let Fragment = O.get("fragment").and_then(Value::as_str).unwrap_or("");

		let mut Url = format!("{}://{}{}", Scheme, Authority, Path);

		if !Query.is_empty() {
			Url.push('?');

			Url.push_str(Query);
		}

		if !Fragment.is_empty() {
			Url.push('#');

			Url.push_str(Fragment);
		}

		Some(Url)
	};

	let RootUriString = match &RootUri {
		Value::String(S) => S.clone(),

		Value::Object(O) => {
			BuildUrlFromComponents(O)
				.or_else(|| O.get("external").and_then(Value::as_str).map(str::to_string))
				.or_else(|| {
					// Last-resort: prepend file:// to a bare path so
					// URLSerializationHelper at least gets a parseable
					// scheme. Never silently emit a relative URL.
					O.get("path")
						.and_then(Value::as_str)
						.filter(|P| P.starts_with('/'))
						.map(|P| format!("file://{}", P))
				})
				.unwrap_or_else(|| "file:///".to_string())
		},

		_ => "file:///".to_string(),
	};

	// Field names must match `SourceControlCreateDTO`'s camelCase wire
	// shape (post-DTO-audit): `id`, `label`, `rootUri`. Earlier revisions
	// passed PascalCase keys here and the trait silently failed with
	// `missing field "id"` because the DTO's serde rename uses camelCase.
	//
	// `handle` is the Cocoon-allocated sequential provider handle (read
	// above from the Parameter). Including it on the wire makes
	// `MountainEnvironment::CreateSourceControl` key its marker maps
	// under the SAME handle that subsequent `register_scm_resource_group`
	// and `update_scm_group` notifications reference - without this,
	// every group update warns "Received group update for unknown
	// provider handle: <H>" because the marker map was keyed by a
	// fresh Mountain-allocated handle Cocoon never sees.
	let CreateData = json!({
		"handle": Handle,
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
	// register signal. Routed through `LogSkyEmit` so `sky-emit` /
	// `grpc` dev-log tags surface delivery success/failure - the
	// fire-and-forget path was previously invisible, making it
	// impossible to tell whether Sky's `Register("sky://scm/register")`
	// listener was hit when the SCM panel stayed empty.
	if let Err(Error) = crate::IPC::SkyEmit::LogSkyEmit(
		Service.ApplicationHandle(),
		"sky://scm/register",
		json!({
			"scmId": &ScmId,
			"label": &Label,
			"rootUri": &RootUriString,
			"extensionId": &ExtensionId,
			"handle": Handle,
		}),
	) {
		dev_log!("grpc", "warn: [Scm] sky://scm/register emit failed for {}: {}", ScmId, Error);
	}

	dev_log!(
		"grpc",
		"[Scm] register provider scmId={} label={} ext={} handle={}",
		ScmId,
		Label,
		ExtensionId,
		Handle
	);
}
