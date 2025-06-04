```sh
src/
├── environment/
│   ├── mod.rs                      // Re-exports MountainEnvironment, declares sub-modules
│   ├── fs_provider.rs              // FsReader, FsWriter impls
│   ├── config_provider.rs          // ConfigProvider, ConfigInspector impls
│   ├── documents_provider.rs       // DocumentProvider impl
│   ├── storage_provider.rs         // StorageProvider impl
│   ├── secrets_provider.rs         // SecretsProvider impl
│   ├── output_provider.rs          // OutputChannelManager impl
│   ├── diagnostics_provider.rs     // DiagnosticsManager impl
│   ├── commands_provider.rs        // CommandExecutor impl
│   ├── workspace_provider.rs       // WorkspaceProvider, WorkspaceEditApplier impls
│   ├── ui_provider.rs              // UiProvider impl
│   ├── ipc_provider.rs             // IpcProvider impl
│   ├── language_features_provider.rs // LanguageFeatureProviderRegistry impl
│   └── utils.rs                    // Shared helpers (map_lock_error, map_io_error, path_to_uri, etc.)
└── (other .rs files like app_state.rs, main.rs, track.rs, etc.)
```
