pub mod AsSlice;
pub mod AsBrotliSlice;
pub mod BrotliLength;

use memmap2::Mmap;

pub struct Struct {
	/// The MemoryMap mapping itself. Keep alive as long as any webview body
	/// references it.
	pub Mapping:Mmap,

	/// Cached MIME from the file extension. Avoids the match arm on the hot
	/// path.
	pub Mime:&'static str,

	/// File size at MemoryMap time. Used for `Content-Length`.
	pub Length:usize,

	/// Optional pre-brotli-compressed sibling (path with `.br` suffix). `None`
	/// if no sibling existed at load time.
	pub Brotli:Option<Mmap>,
}
