//! Source-control-management domain handlers for `CocoonService`.
//! `GitExec::Fn`, `RegisterScmProvider::Fn`, `UpdateScmGroup::Fn`.
/// GitExec handler: executes a git command on behalf of an extension.
pub mod GitExec;

/// RegisterScmProvider handler: registers an SCM provider.
pub mod RegisterScmProvider;

/// UpdateScmGroup handler: updates the resource groups of an SCM provider.
pub mod UpdateScmGroup;
