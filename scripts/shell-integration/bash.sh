#!/usr/bin/env bash
# CodeEditorLand shell integration for bash.
# Injected via `--init-file` when LAND_SHELL_INTEGRATION=1.
# Emits OSC 633 sequences so VS Code's terminal service can track
# command boundaries, exit codes, and the current working directory.

__land_osc() {
	printf '\033]633;%s\007' "$1"
}

# Avoid double-injection if the user's own .bashrc sources us again.
if [[ -n "${LAND_SHELL_INTEGRATION_ACTIVE:-}" ]]; then
	return 0 2> /dev/null || exit 0
fi
export LAND_SHELL_INTEGRATION_ACTIVE=1

# Source the user's .bashrc so aliases, functions, and PATH extensions
# are available. Guard with LAND_SKIP_BASHRC to prevent infinite loops.
if [[ -z "${LAND_SKIP_BASHRC:-}" && -f "$HOME/.bashrc" ]]; then
	export LAND_SKIP_BASHRC=1
	# shellcheck source=/dev/null
	source "$HOME/.bashrc"
fi

# Track whether we are inside a command execution.
__land_in_command=0

__land_preexec() {
	if [[ "$__land_in_command" == "0" ]]; then
		__land_in_command=1
		__land_osc "C"
	fi
}

__land_precmd() {
	local ExitCode=$?
	if [[ "$__land_in_command" == "1" ]]; then
		__land_in_command=0
		__land_osc "D;${ExitCode}"
	fi
	# CWD property - URL-encode spaces; full encoding handled by VS Code.
	__land_osc "P;cwd=$(pwd)"
	# Prompt start
	__land_osc "A"
}

# Bash does not have preexec/precmd hooks natively.  The DEBUG trap fires
# before every command; PROMPT_COMMAND fires before each prompt.
trap '__land_preexec' DEBUG
if [[ -n "${PROMPT_COMMAND:-}" ]]; then
	PROMPT_COMMAND="__land_precmd;${PROMPT_COMMAND}"
else
	PROMPT_COMMAND="__land_precmd"
fi

# Wrap PS1 to emit the "prompt end" marker after the prompt text.
# Using \[ \] to mark the escape as zero-width for readline.
PS1="${PS1}\[\$(printf '\\033]633;B\\007')\]"

export VSCODE_SHELL_INTEGRATION=1
