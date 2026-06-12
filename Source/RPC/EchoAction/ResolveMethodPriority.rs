//! Map a gRPC wire method name to an Echo priority lane.
//!
//! | Wire method                             | Lane   | Reason                              |
//! | --------------------------------------- | ------ | ----------------------------------- |
//! | `FileSystem.ReadFile` / `WriteFile`     | High   | extension UI waits on it            |
//! | `ShowInformationMessage` / `ShowError…` | High   | user-visible                        |
//! | `ExecuteContributedCommand`             | High   | user action                         |
//! | `RegisterCommand` + Register* providers | Normal | activation path                     |
//! | `Configuration.Inspect`                 | Normal | common, not critical                |
//! | `FindFiles` / `FindTextInFiles`         | Low    | long-running                        |
//! | `GitExec`                               | Low    | spawns subprocess                   |
//! | everything else                         | Normal | safe default                        |
use Echo::Task::Priority::Priority as EchoPriority;

/// Map a gRPC wire method name to an Echo scheduler priority lane.
pub fn Fn(Method:&str) -> EchoPriority {
	match Method {
		"FileSystem.ReadFile"
		| "FileSystem.WriteFile"
		| "FileSystem.Stat"
		| "ShowInformationMessage"
		| "ShowWarningMessage"
		| "ShowErrorMessage"
		| "ExecuteContributedCommand"
		| "ShowTextDocument" => EchoPriority::High,

		"FindFiles" | "FindTextInFiles" | "GitExec" | "WatchFile" => EchoPriority::Low,

		_ => EchoPriority::Normal,
	}
}
