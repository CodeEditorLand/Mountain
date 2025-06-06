
// Declares and exports all shim modules for the Cocoon extension host.
// Shims provide implementations for VS Code's internal `ExtHost` services,
// adapting them to the Mountain/Cocoon architecture.

#![allow(non_snake_case, non_camel_case_types)]

// Sub-modules for each shimmed service and utility.
mod ApiDeprecation;
mod Authentication;
mod BaseShim; // Foundational shim class
mod Clipboard;
mod Command; // The `vscode.Command` data class
mod Commands;
mod Configuration;
mod Debug;
mod Diagnostic;
mod Dialog;
mod Disposable; // The `vscode.Disposable` utility class
mod Document;
mod EnablementService;
mod Env;
mod Extension;
mod FileSystemApi;
mod FileSystemInfo;
mod Fs; // Obsolete fs shim
mod FsModuleShimFactory; // Obsolete
mod HostKindPicker;
mod HostUtil;
mod Language;
mod LanguageFeature;
mod LanguageModel;
mod Localization;
mod Log;
mod ManagedSocket;
mod Message;
mod NodeModuleShimFactory;
mod Os;
mod OutputChannel;
mod Process;
mod ProposedApi;
mod QuickInput;
mod SecretState;
mod Storage;
mod StoragePath;
mod Task;
mod Telemetry;
mod Terminal;
mod TypeConverter; // Module for all type converters
mod Ui; // Obsolete
mod UriTransformer;
mod Vscode; // The shimmed vscode API module
mod WindowPart;
mod Workspace;

// Re-exporting primary shim classes for use in DI and other parts of the
// application.
// pub use self::Ui::ShimExtHostUiAndEnv; // Obsolete, not exported
pub use self::{
	ApiDeprecation::ShimExtHostApiDeprecationService,
	Authentication::ShimExtHostAuthentication,
	BaseShim::{BaseCocoonShim, ILogServiceForShim, IRpcProtocolServiceAdapter},
	Clipboard::ShimExtHostClipboardService,
	Command::Command,
	Commands::ShimExtHostCommands,
	Configuration::ShimExtHostConfiguration,
	Debug::ShimExtHostDebugService,
	Diagnostic::ShimDiagnosticsService,
	Dialog::ShimExtHostDialogService,
	Disposable::Disposable,
	Document::CocoonDocumentService,
	EnablementService::ShimExtensionEnablementService,
	Env::ShimExtHostEnvService,
	Extension::ShimExtHostExtensions,
	FileSystemApi::ShimFileSystemApi,
	FileSystemInfo::ShimExtHostFileSystemInfo,
	Fs::default as FsShimInstance,
	FsModuleShimFactory::FsModuleShimFactory,
	HostKindPicker::ShimExtensionHostKindPicker,
	HostUtil::ShimHostUtils,
	Language::ShimLanguages,
	LanguageFeature::ShimLanguageFeatures,
	LanguageModel::ShimExtHostLanguageModels,
	Localization::ShimExtHostLocalizationService,
	Log::{ShimLogService, ShimLoggerService},
	ManagedSocket::ShimExtHostManagedSockets,
	Message::ShimExtHostMessageService,
	NodeModuleShimFactory::NodeModuleShimFactory,
	Os::default as OsShimInstance,
	OutputChannel::ShimOutputService,
	Process::default as ProcessShimInstance,
	ProposedApi::ShimExtensionsProposedApi,
	QuickInput::ShimExtHostQuickInputService,
	SecretState::ShimExtHostSecretState,
	Storage::ShimExtHostStorage,
	StoragePath::ShimExtensionStoragePaths,
	Task::ShimExtHostTaskService,
	Telemetry::ShimExtHostTelemetry,
	Terminal::ShimExtHostTerminalService,
	UriTransformer::ShimUriTransformerService,
	WindowPart::ShimExtHostWindowPartsService,
	Workspace::ShimExtHostWorkspace,
};
