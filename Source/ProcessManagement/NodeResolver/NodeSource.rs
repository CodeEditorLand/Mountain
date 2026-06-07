//! Where the resolved Node binary came from. Ordered by preference (override
//! first, PATH last). `AsLabel` returns the lowercase ident used in log
//! lines.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Enum {

	/// `Pick` environment variable.
	Override,

	/// Shipped with Mountain - `Resources/Node/bin/node` or dev-tree
	/// equivalent.
	Shipped,

	/// fnm's `current/bin/node`.
	Fnm,

	/// Volta's `tools/image/node/<version>/bin/node`.
	Volta,

	/// asdf's `shims/node` - resolves via `.tool-versions`.
	Asdf,

	/// nvm's `versions/node/<default>/bin/node`.
	Nvm,

	/// Homebrew - `/opt/homebrew/bin/node` (Apple Silicon) or
	/// `/usr/local/bin/node` (Intel macOS / Linuxbrew).
	Homebrew,

	/// PATH-resolved `node` - last-resort fallback.
	Path,
}

impl Enum {

	pub fn AsLabel(self) -> &'static str {
		match self {
			Self::Override => "override",

			Self::Shipped => "shipped",

			Self::Fnm => "fnm",

			Self::Volta => "volta",

			Self::Asdf => "asdf",

			Self::Nvm => "nvm",

			Self::Homebrew => "homebrew",

			Self::Path => "path",
		}
	}
}
