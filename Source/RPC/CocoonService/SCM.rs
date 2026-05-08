#![allow(non_snake_case)]

//! Source-control-management domain handlers for `CocoonService`.
//! `RegisterScmProvider::Fn`, `UpdateScmGroup::Fn`, `GitExec::Fn`.

pub mod GitExec;

pub mod RegisterScmProvider;

pub mod UpdateScmGroup;
