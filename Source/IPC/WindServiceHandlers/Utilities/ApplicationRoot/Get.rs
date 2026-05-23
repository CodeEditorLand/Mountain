
//! Returns the cached static application root path.

pub fn Fn() -> Option<String> { super::ROOT.get().cloned() }
