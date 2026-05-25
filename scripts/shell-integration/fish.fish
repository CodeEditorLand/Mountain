#!/usr/bin/env fish
# CodeEditorLand shell integration for fish.
# Sourced via `--init-command` when LAND_SHELL_INTEGRATION=1.

function __land_osc
    printf '\033]633;%s\007' $argv[1]
end

if set -q LAND_SHELL_INTEGRATION_ACTIVE
    exit 0
end
set -gx LAND_SHELL_INTEGRATION_ACTIVE 1

function __land_on_preexec --on-event fish_preexec
    __land_osc "C"
end

function __land_on_postexec --on-event fish_postexec
    __land_osc "D;$status"
    __land_osc "P;cwd=$(pwd)"
    __land_osc "A"
end

function __land_on_prompt --on-event fish_prompt
    __land_osc "B"
end

set -gx VSCODE_SHELL_INTEGRATION 1
