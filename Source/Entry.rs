// File: Entry.rs
// Main application entry point and setup for the Mountain backend.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{path::PathBuf, sync::Arc};

use colored::Colorize;
use log::{debug, error, info, trace, warn};
use mountain::{
	AppState, // Corrected: Assuming this is the actual name of the AppState struct
	Environment::MountainEnvironment,
	Handlers::{self, SkyCommands as ActualSkyCommands, SkyIpcBridge, SkyUiResponses},
	Mist, // For WebSocket server if enabled
	Rpc,  // For gRPC server setup if used directly
	Runtime::AppRuntime,
	Track,
	Vine, // For gRPC if enabled
};
use tauri::{AppHandle, Manager, RunEvent, WebviewWindowBuilder, Wry};

mod LoggingSetup {
	pub fn InitializeLogger() {
		#[cfg(debug_assertions)]
		{
			env_logger::Builder::new()
				.filter_level(log::LevelFilter::Trace)
				.format(|Buffer, Record| {
					use std::io::Write;
					writeln!(
						Buffer,
						"[{}] [{}]: {}",
						"Mountain".red(),
						match Record.level() {
							log::Level::Error => "ERROR".red().bold(),
							log::Level::Warn => "WARN".yellow().bold(),
							log::Level::Info => "INFO".green(),
							log::Level::Debug => "DEBUG".blue(),
							log::Level::Trace => "TRACE".magenta(),
						},
						Record.args()
					)
				})
				.try_init()
				.expect("Failed to initialize env_logger. Another logger might be active.");
		}
		#[cfg(not(debug_assertions))]
		{
			env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
				.format_timestamp_millis()
				.init();
		}
		log::info!("[LoggerInit] Mountain logger initialized.");
	}
}

#[tokio::main]
async fn main() {
	LoggingSetup::InitializeLogger();
	info!("[MountainMain] Starting up Land Editor (Mountain backend)...");

	let mut Builder = tauri::Builder::default();

	#[cfg(any(windows, target_os = "linux"))]
	{
		Builder = Builder.any_thread();
	}

	Builder
		.manage(AppState::default())
		.setup(|App| {
			info!("[MountainSetup] Tauri setup hook executing...");
			let AppHandle = App.handle().clone();

			let MountainEnvArc = Arc::new(MountainEnvironment::New(AppHandle.clone()));
			let AppRuntimeArc = Arc::new(AppRuntime::New(MountainEnvArc.clone()));
			AppHandle.manage(AppRuntimeArc.clone());
			info!("[MountainSetup] MountainEnvironment and AppRuntime created and managed.");

			let PostSetupAppHandle = AppHandle.clone();
			tauri::async_runtime::spawn(async move {
				info!("[MountainSetupTask] Starting AppState post-AppHandle initialization...");
				let AppStateInstance = PostSetupAppHandle.state::<AppState>();

				{
					let mut ResolvedScanPaths:Vec<PathBuf> = Vec::new();
					let PathResolver = PostSetupAppHandle.path_resolver();

					if let Some(BuiltinExtensionsPath) = PathResolver.resolve_resource("extensions/builtin") {
						if BuiltinExtensionsPath.is_dir() {
							info!(
								"[MountainSetupTask] Adding builtin extension scan path: {}",
								BuiltinExtensionsPath.display()
							);
							ResolvedScanPaths.push(BuiltinExtensionsPath);
						} else {
							warn!(
								"[MountainSetupTask] Resolved builtin extension path '{}' is not valid. Skipping.",
								BuiltinExtensionsPath.display()
							);
						}
					} else {
						warn!("[MountainSetupTask] Could not resolve 'extensions/builtin' resource path.");
					}

					if let Some(AppDataPath) = PathResolver.app_data_dir() {
						let UserExtensionsPath = AppDataPath.join("extensions");
						if UserExtensionsPath.is_dir() {
							info!(
								"[MountainSetupTask] Adding user extension scan path: {}",
								UserExtensionsPath.display()
							);
							ResolvedScanPaths.push(UserExtensionsPath);
						} else {
							trace!(
								"[MountainSetupTask] User extension path '{}' does not exist. Skipping.",
								UserExtensionsPath.display()
							);
						}
					} else {
						warn!("[MountainSetupTask] Could not resolve app data directory for user extensions.");
					}

					let mut ScanPathsGuard = AppStateInstance
						.ExtensionScanPaths
						.lock()
						.expect("Failed to lock ExtensionScanPaths");
					ScanPathsGuard.clear();
					ScanPathsGuard.extend(ResolvedScanPaths);
					debug!("[MountainSetupTask] Final extension scan paths: {:?}", *ScanPathsGuard);
				}

				AppStateInstance.ScanExtensionsAndPopulateState().await;

				let mut ProposedApisGuard = AppStateInstance
					.EnabledProposedApis
					.lock()
					.expect("Failed to lock EnabledProposedApis");
				ProposedApisGuard.insert(
					"*".to_string(),
					vec!["testProposedApi".to_string(), "workspaceTrust".to_string()],
				);
				info!(
					"[MountainSetupTask] Enabled proposed APIs configured: {}",
					ProposedApisGuard.len()
				);

				match Handlers::Config::LoadAndMergeConfigurationsInternal(&PostSetupAppHandle, &AppStateInstance).await
				{
					Ok(MergedConfig) => {
						AppStateInstance
							.Configuration
							.lock()
							.expect("Failed to lock Configuration for init")
							.Update(MergedConfig);
						info!("[MountainSetupTask] Initial merged configuration loaded.");
					},
					Err(Error) => {
						error!(
							"[MountainSetupTask] CRITICAL: Failed to load initial configurations: {}. App may \
							 malfunction.",
							Error
						);
					},
				}

				if let Some(AppDataDirForMemento) = PostSetupAppHandle.path_resolver().app_data_dir() {
					if let Err(Error) = AppStateInstance.UpdateWorkspaceMementoPathAndReload(&AppDataDirForMemento) {
						error!("[MountainSetupTask] Failed to init workspace memento: {}", Error);
					}
				} else {
					warn!("[MountainSetupTask] App data dir unavailable for workspace memento init.");
				}
				info!("[MountainSetupTask] AppState post-AppHandle initialization complete.");
			});

			let ProtocolAppHandle = AppHandle.clone();
			if let Err(Error) = App.protocol().register("vscode", move |IpcRequest| {
				debug!("[MountainSetupProtocol] Received vscode:// request: {}", IpcRequest.uri());
				Handlers::Protocol::HandleCustomUriSchemeRequest(IpcRequest, ProtocolAppHandle.clone())
			}) {
				error!("[MountainSetup] CRITICAL: Failed to register 'vscode://' protocol: {}", Error);
			} else {
				info!("[MountainSetup] 'vscode://' protocol registered.");
			}

			let RpcSetupRuntime = AppHandle.state::<Arc<AppRuntime>>().inner().clone();
			Rpc::SetupMountainRpcServer(AppHandle.clone(), RpcSetupRuntime);
			info!("[MountainSetup] RPC Server endpoint logic ready for Vine (gRPC).");

			#[cfg(feature = "extension_host_cocoon")]
			{
				info!("[MountainSetup] 'extension_host_cocoon' ENABLED. Spawning Cocoon sidecar task...");
				let CocoonLaunchAppHandle = AppHandle.clone();
				let CocoonGrpcServerAddress = AppState::default().GlobalMementoPath // Placeholder for actual config
                    .parent().unwrap().join("cocoon_grpc.socket").to_string_lossy().to_string();

				tauri::async_runtime::spawn(async move {
					Handlers::ProcessManagement::LaunchAndManageCocoonSidecar(
						CocoonLaunchAppHandle,
						CocoonGrpcServerAddress,
					)
					.await;
				});
			}
			#[cfg(not(feature = "extension_host_cocoon"))]
			{
				info!("[MountainSetup] 'extension_host_cocoon' DISABLED. Cocoon sidecar will not launch.");
			}

			#[cfg(feature = "mist_native")]
			{
				info!("[MountainSetup] 'mist_native' ENABLED. Spawning Mist WebSocket server task...");
				let MistAppHandle = AppHandle.clone();
				tauri::async_runtime::spawn(async move {
					if let Err(Error) = Mist::StartWebSocketServer(MistAppHandle).await {
						error!("[MistServerStartup] Mist WebSocket server failed: {}", Error);
					}
				});
			}
			#[cfg(not(feature = "mist_native"))]
			{
				info!("[MountainSetup] 'mist_native' DISABLED. Mist WebSocket server will not start.");
			}

			info!("[MountainSetup] Initializing main application window...");
			let mut WindowBuilder = WebviewWindowBuilder::new(
				App,
				"Application",
				tauri::WebviewUrl::App(std::path::PathBuf::from("Application/index.html")),
			)
			.use_https_scheme(true)
			.zoom_hotkeys_enabled(true)
			.browser_extensions_enabled(false);

			#[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
			{
				WindowBuilder = WindowBuilder
					.title("Land Editor")
					.maximized(true)
					.decorations(false)
					.shadow(true);
			}

			match WindowBuilder.build() {
				Ok(WindowInstance) => {
					info!("[MountainSetup] Main application window created.");
					#[cfg(all(debug_assertions, not(any(target_os = "android", target_os = "ios"))))]
					{
						WindowInstance.open_devtools();
						info!("[MountainSetup] Developer tools opened for main window.");
					}
				},
				Err(Error) => {
					error!("[MountainSetup] Main window build failed: {:?}", Error);
					panic!("Main application window build failed: {:?}", Error);
				},
			};
			info!("[MountainSetup] Setup hook complete.");
			Ok(())
		})
		.plugin(tauri_plugin_dialog::init())
		.invoke_handler(tauri::generate_handler![
			Track::DispatchCommand,
			SkyUiResponses::SkyResolvesUiRequest,
			Track::MountainGetWorkbenchConfiguration,
			ActualSkyCommands::MountainSetZoomLevel,
			ActualSkyCommands::MountainFetchShellEnv,
			ActualSkyCommands::MountainGetProcessMemoryInfo,
			SkyIpcBridge::MountainIpcBridgeSend,
			SkyIpcBridge::MountainIpcBridgeInvoke,
			Track::MountainRequestHover,
			Track::MountainRequestCompletions,
			Track::MountainResolveCompletionItem,
			Track::MountainRequestCodeActions,
			Track::MountainResolveCodeAction,
			Track::MountainRequestCodeLenses,
			Track::MountainResolveCodeLens,
			Track::MountainRequestDocumentSymbols,
			Track::MountainRequestWorkspaceSymbols,
			Track::MountainRequestSignatureHelp,
			Track::MountainRequestReferences,
			Track::MountainPrepareRename,
			Track::MountainProvideRenameEdits,
			Track::MountainApplyWorkspaceEdit,
			Track::MountainRequestDocumentFormatting,
			Track::MountainRequestDocumentHighlights,
			Track::MountainRequestDocumentLinks,
			Track::MountainResolveDocumentLink,
			Track::MountainRequestFoldingRanges,
			Track::MountainRequestSelectionRanges,
			Track::MountainRequestLinkedEditingRanges,
			Track::MountainRequestDocumentSemanticTokens,
			Track::MountainRequestDocumentSemanticTokensEdits,
			Track::MountainPrepareCallHierarchy,
			Track::MountainProvideCallHierarchyIncoming,
			Track::MountainProvideCallHierarchyOutgoing,
			Track::MountainPrepareTypeHierarchy,
			Track::MountainProvideTypeHierarchySupertypes,
			Track::MountainProvideTypeHierarchySubtypes,
			Track::MountainRequestInlayHints,
			Track::MountainResolveInlayHint
		])
		.on_window_event(|Event| {
			match Event.event() {
				tauri::WindowEvent::CloseRequested { api, .. } => {
					info!("[MountainWindowEvent] Close requested for window: {}", Event.window().label());
					warn!(
						"[MountainWindowEvent] CloseRequested: Graceful shutdown not fully implemented. Defaulting to \
						 close."
					);
				},
				tauri::WindowEvent::Destroyed => {
					info!("[MountainWindowEvent] Window destroyed: {}", Event.window().label());
				},
				_ => {
					trace!(
						"[MountainWindowEvent] Other event on '{}': {:?}",
						Event.window().label(),
						Event.event()
					);
				},
			}
		})
		.build(tauri::generate_context!())
		.expect("FATAL: Error building Mountain Tauri application")
		.run(|AppHandleRun, Event| {
			match Event {
				RunEvent::ExitRequested { api, .. } => {
					info!("[MountainRunEvent] Application exit requested.");
					warn!(
						"[MountainRunEvent] ExitRequested: Pre-exit cleanup not fully implemented. Proceeding with \
						 default exit."
					);
				},
				RunEvent::Exit => {
					info!("[MountainRunEvent] Application exited.");
				},
				RunEvent::Ready => {
					info!("[MountainRunEvent] Application ready.");
				},
				_ => {
					trace!("[MountainRunEvent] Other run event: {:?}", Event);
				},
			}
		});

	info!("[MountainMain] Application event loop finished or error during run.");
}
