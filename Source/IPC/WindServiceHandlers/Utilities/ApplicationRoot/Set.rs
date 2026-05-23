#![allow(non_snake_case)]

//! Sets the static application root path (once, at startup).

pub fn Fn(Path:String) { let _ = super::ROOT.set(Path); }
