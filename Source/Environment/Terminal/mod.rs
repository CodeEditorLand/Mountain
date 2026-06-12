//! Terminal environment support for the integrated terminal.
//!
//! Covers shell integration: injecting per-shell startup hooks (bash, zsh,
//! fish) so the workbench receives OSC 633 sequences for command tracking
//! and current-working-directory reporting.

/// Shell integration script injection — embeds OSC-633 hooks into bash, zsh,
/// and fish startup sequences for command tracking and CWD reporting.
pub mod ShellIntegration;
