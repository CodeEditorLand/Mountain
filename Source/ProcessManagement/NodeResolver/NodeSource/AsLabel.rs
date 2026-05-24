//! `NodeSource::AsLabel`

use super::Struct;


pub fn Fn(self) -> &'static str {
		match self {
			Struct::Override => "override",

			Struct::Shipped => "shipped",

			Struct::Fnm => "fnm",

			Struct::Volta => "volta",

			Struct::Asdf => "asdf",

			Struct::Nvm => "nvm",

			Struct::Homebrew => "homebrew",

			Struct::Path => "path",
		}
	}
