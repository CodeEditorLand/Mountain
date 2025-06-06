

// File: Handler/SkyConfiguration/SkyConfiguration.rs
// Defines the logic for building the initial ISandboxConfiguration DTO,
// which provides the Sky (frontend) with necessary environment and bootstrap data.

#![allow(non_snake_case, non_camel_case_types)] 

use std::collections::HashMap;
use std::env;
use std::path::PathBuf;

use chrono::Utc;
use log::{debug, error, info, warn};
use tauri::utils::platform::current_exe;
use tauri::{AppHandle, Manager, Wry};
use uuid::Uuid;

use crate::AppState;
use crate::Handlers::SkyDtos::{
    NlsConfigurationDto,
    ProcessVersionsDto,
    ProductConfigurationDto,
    SandboxConfigurationDto,
};

/// Maps the `log::LevelFilter` to the corresponding numeric level expected by VS Code's frontend.
fn MapLogLevelFilterToVSCodeLevel(Filter: log::LevelFilter) -> u32 {
    match Filter {
        log::LevelFilter::Trace => 0,
        log::LevelFilter::Debug => 1,
        log::LevelFilter::Info => 2,
        log::LevelFilter::Warn => 3,
        log::LevelFilter::Error => 4,
        log::LevelFilter::Off => 6, // Level 5 is Critical, which we map to Error. 6 is Off.
    }
}

/// A heuristic to get the webview version string based on the operating system.
fn GetWebviewVersionHeuristic() -> String {
    if cfg!(target_os = "windows") {
        "Edge WebView2/Unknown".to_string()
    } else if cfg!(target_os = "macos") {
        "WebKit/Unknown (macOS)".to_string()
    } else if cfg!(target_os = "linux") {
        "WebKitGTK/Unknown (Linux)".to_string()
    } else {
        "Unknown Webview".to_string()
    }
}

/// Converts a `PathBuf` to a file URI string.
fn PathBufToUriString(Path: &PathBuf) -> String {
    url::Url::from_file_path(Path)
        .map(|UrlInstance| UrlInstance.to_string())
        .unwrap_or_else(|_| {
            warn!("[SkyConfig] Failed to convert path to file URL: {}. Using lossy string.", Path.display());
            format!("file:
        })
}

/// Constructs the full `SandboxConfigurationDto` for the frontend.
pub fn BuildSandboxConfiguration(ApplicationHandle: &AppHandle<Wry>) -> SandboxConfigurationDto {
    info!("[SkyConfig] Building ISandboxConfiguration for Sky frontend...");

    let AppStateInstance = ApplicationHandle.state::<AppState>();
    let PathResolver = ApplicationHandle.path_resolver();
    let PackageInformation = ApplicationHandle.package_info();

    let PlatformString = match env::consts::OS {
        "windows" => "win32",
        "macos" => "darwin",
        "linux" => "linux",
        _ => "unknown",
    }
    .to_string();

    let ArchitectureString = match env::consts::ARCH {
        "x86_64" => "x64",
        "aarch64" => "arm64",
        "x86" => "ia32",
        _ => "unknown",
    }
    .to_string();

    let HomeDirectoryPath = PathResolver.home_dir().unwrap_or_else(|| PathBuf::from("/fallback/home"));
    let TemporaryDirectoryPath = env::temp_dir();
    let UserDataDirectoryPath = PathResolver.app_data_dir().unwrap_or_else(|| PathBuf::from("/fallback/appdata"));
    let ResourcesDirectoryPath = PathResolver.resource_dir().unwrap_or_default();
    let ExecutablePath = current_exe().unwrap_or_default();

    let AppRootUri = ApplicationHandle
        .config()
        .build
        .dev_url
        .as_ref()
        .map(|DevelopmentUrlValue| format!("{}/Static/Application/", DevelopmentUrlValue.as_str().trim_end_matches('/')))
        .unwrap_or_else(|| "app:
    
    let CurrentLanguage = env::var("LANG")
        .map(|LocaleString| LocaleString.split('.').next().unwrap_or(&LocaleString).split('_').next().unwrap_or(&LocaleString).to_lowercase())
        .unwrap_or_else(|_| "en".to_string());

    let VersionsDto = ProcessVersionsDto {
        AppName: Some(PackageInformation.name.clone()),
        AppVersion: Some(PackageInformation.version.to_string()),
        TauriVersion: Some(tauri::VERSION.to_string()),
        WebviewRuntimeVersion: Some(GetWebviewVersionHeuristic()),
    };

    let ProductConfigurationDto = ProductConfigurationDto {
        NameShort: Some(PackageInformation.name.chars().take(8).collect::<String>().to_uppercase()),
        NameLong: Some(format!("{} Code Editor", PackageInformation.name)),
        ApplicationName: Some(PackageInformation.name.to_lowercase().replace(' ', "-")),
        Version: Some(PackageInformation.version.to_string()),
        Commit: Some(env::var("SOURCE_COMMIT_HASH").unwrap_or_else(|_| "development".to_string())),
        Date: Some(Utc::now().to_rfc3339()),
        DataFolderName: Some(format!(".{}", PackageInformation.name.to_lowercase())),
        EmbedderIdentifier: Some("desktop".to_string()),
        AdditionalProperties: HashMap::new(),
    };

    let WorkspaceIdentifierForBackup = AppStateInstance.GetWorkspaceIdentifier().unwrap_or_else(|Error| {
        warn!("[SkyConfig] Failed to get workspace ID for backup path: {}. Using default.", Error);
        "default_workspace".to_string()
    });
    let BackupPathUriString = PathBufToUriString(&UserDataDirectoryPath.join("Backups").join(WorkspaceIdentifierForBackup));

    SandboxConfigurationDto {
        WindowIdentifier: ApplicationHandle.webview_window("main")
            .map(|Window| Window.label().to_string())
            .unwrap_or_else(|| "main_window_fallback".to_string()),
        MachineIdentifier: ApplicationHandle.manager().instance_id().to_string(),
        SessionIdentifier: Uuid::new_v4().to_string(),
        SqmIdentifier: Some(ApplicationHandle.manager().instance_id().to_string()),
        LogLevel: MapLogLevelFilterToVSCodeLevel(log::max_level()),
        UserEnvironment: HashMap::new(),
        AppRoot: AppRootUri,
        AppName: PackageInformation.name.clone(),
        AppUriScheme: PackageInformation.name.to_lowercase(),
        AppLanguage: CurrentLanguage.clone(),
        AppHost: "desktop".to_string(),
        ProductQuality: Some(if cfg!(debug_assertions) { "development".to_string() } else { "stable".to_string() }),
        Platform: PlatformString,
        Architecture: ArchitectureString,
        Versions: VersionsDto,
        ExecutablePath: ExecutablePath.to_string_lossy().into_owned(),
        ZoomLevel: Some(0.0),
        HomeDirectory: PathBufToUriString(&HomeDirectoryPath),
        TemporaryDirectory: PathBufToUriString(&TemporaryDirectoryPath),
        UserDataDirectory: PathBufToUriString(&UserDataDirectoryPath),
        BackupPath: Some(BackupPathUriString),
        CrashReporterIdentifier: Some(Uuid::new_v4().to_string()),
        Nls: NlsConfigurationDto {
            MessageMap: HashMap::new(),
            Language: CurrentLanguage,
            AvailableLanguageMap: HashMap::from([("en".to_string(), "English".to_string())]),
            Pseudo: Some(false),
        },
        ProductConfiguration: ProductConfigurationDto,
        VsCodeCurrentWorkingDirectory: Some(env::current_dir().map(|p| p.to_string_lossy().into_owned()).unwrap_or_default()),
        ResourcesPath: ResourcesDirectoryPath.to_string_lossy().into_owned(),
        AdditionalProperties: HashMap::new(),
    }
}
