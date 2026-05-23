
//! Recognise known-optional probe paths so `stat ENOENT`
//! lines for them downgrade to debug-once instead of full
//! error noise. The list is the union of:
//!
//! - VS Code / Copilot / Claude / vim probe paths.
//! - Per-extension state probes (`globalStorage`, `workspaceStorage`, sqlite
//!   state files).
//! - First-run user-config files (`tasks.json`, `mcp.json`, `keybindings.json`,
//!   …) lazy-created on first write.
//! - `vscode://schemas-associations/` virtual-resource probes.
//! - External-editor / vim-config detection paths used by "Open With…" pickers.

const BENIGN_ENOENT_SUBSTRINGS:&[&str] = &[
	"/.claude",
	"/.vscode",
	".claude/agents",
	".claude/settings.json",
	".claude/settings.local.json",
	".copilot/agents",
	".github/copilot",
	".github/agents",
	".vscode/settings.json",
	".vscode/launch.json",
	".vscode/extensions.json",
	".vscode/tasks.json",
	".vscode/mcp.json",
	".mcp.json",
	"agentPlugins",
	"agent-plugins",
	"chatEditingSessions",
	"chatSessions",
	"machineid",
	"terminalSuggestGlobalsCacheV2.json",
	"globalStorage",
	"/User/tasks.json",
	"/User/mcp.json",
	"/User/snippets",
	"/User/keybindings.json",
	"aiGeneratedWorkspaces.json",
	"/.git/config",
	"chatLanguageModels.json",
	"configurationDefaultsOverrides",
	"vscode-chat-images",
	"/output_20",
	"/network.log",
	"/renderer.log",
	"/views.log",
	"/notebook.rendering.log",
	"vscode://schemas-associations/",
	"vscodevim.vim/.registers",
	"/User/globalStorage/",
	"/chatEditingSessions/",
	"/User/prompts",
	"languageDetectionWorkerCache.json",
	"/Applications/Eclipse IDE.app",
	"/Applications/Eclipse.app",
	"/Applications/IntelliJ IDEA.app",
	"/Applications/IntelliJ IDEA CE.app",
	"/Applications/Sublime Text.app",
	"/Applications/Notepad++.app",
	"/Applications/Visual Studio Code.app",
	"/Applications/Xcode.app",
	"/.config/nvim/init.lua",
	"/.config/nvim/init.vim",
	"/.vimrc",
	"/.gvimrc",
	"/state.vscdb",
	"/state.vscdb-journal",
	"/User/workspaceStorage/",
	"/globalStorage/eamodio.gitlens",
	"/globalStorage/GitHub.copilot",
	"/globalStorage/GitHub.copilot-chat",
	"/globalStorage/Anthropic.claude-code",
	"/globalStorage/RooVeterinaryInc.roo-cline",
	".registers",
	"/Sky/Target/product.json",
	"/Output/Target/product.json",
];

pub fn Fn(Path:&str) -> bool { BENIGN_ENOENT_SUBSTRINGS.iter().any(|Needle| Path.contains(Needle)) }
