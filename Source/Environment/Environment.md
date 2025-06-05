```sh
src/
├── environment/
│   ├── commands_provider.rs        // CommandExecutor impl
│   ├── config_provider.rs          // ConfigProvider, ConfigInspector impls
│   ├── diagnostics_provider.rs     // DiagnosticsManager impl
│   ├── documents_provider.rs       // DocumentProvider impl
│   ├── fs_provider.rs              // FsReader, FsWriter impls
│   ├── ipc_provider.rs             // IpcProvider impl
│   ├── language_features_provider.rs // LanguageFeatureProviderRegistry impl
│   ├── mod.rs                      // Re-exports MountainEnvironment, declares sub-modules
│   ├── output_provider.rs          // OutputChannelManager impl
│   ├── secrets_provider.rs         // SecretsProvider impl
│   ├── storage_provider.rs         // StorageProvider impl
│   ├── ui_provider.rs              // UiProvider impl
│   ├── utils.rs                    // Shared helpers (map_lock_error, map_io_error, path_to_uri, etc.)
│   └── workspace_provider.rs       // WorkspaceProvider, WorkspaceEditApplier impls
└── (other .rs files like app_state.rs, main.rs, track.rs, etc.)
```
