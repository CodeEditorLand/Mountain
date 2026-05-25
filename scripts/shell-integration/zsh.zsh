#!/usr/bin/env zsh
# CodeEditorLand shell integration for zsh.
# Injected by setting ZDOTDIR to a temp dir whose .zshrc sources this file,
# then the user's original ~/.zshrc, when LAND_SHELL_INTEGRATION=1.

__land_osc() {
	printf '\033]633;%s\007' "$1"
}

if [[ -n "${LAND_SHELL_INTEGRATION_ACTIVE:-}" ]]; then
	return 0
fi
export LAND_SHELL_INTEGRATION_ACTIVE=1

# Re-source the user's .zshrc after integration hooks are installed.
if [[ -z "${LAND_SKIP_ZSHRC:-}" && -f "${LAND_ORIG_ZDOTDIR:-$HOME}/.zshrc" ]]; then
	export LAND_SKIP_ZSHRC=1
	# shellcheck source=/dev/null
	source "${LAND_ORIG_ZDOTDIR:-$HOME}/.zshrc"
fi

autoload -Uz add-zsh-hook

__land_preexec() {
	__land_osc "C"
}

__land_precmd() {
	local ExitCode=$?
	__land_osc "D;${ExitCode}"
	__land_osc "P;cwd=$(pwd)"
	__land_osc "A"
}

add-zsh-hook preexec __land_preexec
add-zsh-hook precmd __land_precmd

# Append the prompt-end marker to PS1.
PS1="${PS1}%{$(__land_osc "B")%}"

export VSCODE_SHELL_INTEGRATION=1
