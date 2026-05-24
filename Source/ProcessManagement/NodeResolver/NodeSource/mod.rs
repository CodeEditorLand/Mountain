pub mod AsLabel;

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

#[derive(Debug, Clone)]
pub struct Struct;
