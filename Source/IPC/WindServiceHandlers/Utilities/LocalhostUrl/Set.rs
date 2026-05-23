//! Sets the localhost plugin base URL (once, at startup).

pub fn Fn(Url:String) { let _ = super::URL.set(Url); }
