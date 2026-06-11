//! Fire-and-forget startup extension activation: the `*` burst, webview
//! panel restore, open document/terminal seeding, `workspaceContains:`
//! scans, and the deferred `onStartupFinished` event.

use std::{sync::Arc, time::Duration};

use tokio::time::sleep;

use crate::{Environment::MountainEnvironment::MountainEnvironment, dev_log};

pub(crate) fn Fn(SideCarIdentifier:&str, Environment:&Arc<MountainEnvironment>) {
	// Trigger startup extension activation. Cocoon is fully reactive -
	// it won't activate any extensions until Mountain tells it to.
	// Fire-and-forget: don't block on activation, and don't fail init if it errors.
	//
	// Stock VS Code fires a cascade of activation events at boot:
	//   1. `*` - unconditional "activate anything that contributes *"
	//   2. `onStartupFinished` - queued extensions whose start may be deferred
	//      until after the first frame renders
	//   3. `workspaceContains:<pattern>` for each pattern any extension
	//      contributes, fired per matching workspace folder
	//
	// Previously only `*` fired, which meant a large class of extensions
	// that gate on `workspaceContains:package.json`, `onStartupFinished`,
	// or similar events never activated without user interaction. The
	// added bursts below bring startup coverage in line with stock.
	let SideCarId = SideCarIdentifier.to_string();

	let EnvironmentForActivation = Environment.clone();

	tauri::async_runtime::spawn(async move {
		// Small delay to let Cocoon finish processing the init response
		sleep(Duration::from_millis(500)).await;

		crate::dev_log!("exthost", "Sending $activateByEvent(\"*\") to Cocoon");

		if let Err(Error) = crate::Vine::Client::SendRequest::Fn(
			&SideCarId,
			"$activateByEvent".to_string(),
			serde_json::json!({ "activationEvent": "*" }),
			30_000,
		)
		.await
		{
			dev_log!("cocoon", "warn: [CocoonManagement] $activateByEvent(\"*\") failed: {}", Error);

			return;
		}

		dev_log!("cocoon", "[CocoonManagement] Startup extensions activation (*) triggered");

		super::RestoreWebviewPanels::Fn(&SideCarId, &EnvironmentForActivation).await;

		super::SeedOpenDocuments::Fn(&SideCarId, &EnvironmentForActivation).await;

		super::SeedOpenTerminals::Fn(&SideCarId, &EnvironmentForActivation).await;

		super::FireWorkspaceContainsEvents::Fn(&SideCarId, &EnvironmentForActivation).await;

		// Phase 3: onStartupFinished. Fire after the `*` burst has had a
		// moment to complete so late-binding extensions layered on top
		// of startup contributions resolve in the expected order.
		sleep(Duration::from_millis(2_000)).await;

		if let Err(Error) = crate::Vine::Client::SendRequest::Fn(
			&SideCarId,
			"$activateByEvent".to_string(),
			serde_json::json!({ "activationEvent": "onStartupFinished" }),
			30_000,
		)
		.await
		{
			dev_log!(
				"cocoon",
				"warn: [CocoonManagement] $activateByEvent(onStartupFinished) failed: {}",
				Error
			);
		} else {
			dev_log!("cocoon", "[CocoonManagement] onStartupFinished activation triggered");
		}

		super::FireRootConfigActivationEvents::Fn(&EnvironmentForActivation).await;
	});
}
