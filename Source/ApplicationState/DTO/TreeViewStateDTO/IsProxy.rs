//! `TreeViewStateDTO::IsProxy`

use super::Struct;
use std::sync::Arc;
use CommonLibrary::TreeView::TreeViewProvider::TreeViewProvider;

pub fn Fn(This:&Struct) -> bool { This.SideCarIdentifier.is_some() }
