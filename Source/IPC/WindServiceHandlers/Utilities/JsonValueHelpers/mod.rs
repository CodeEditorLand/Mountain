//! `JsonValueHelpers` - one `pub fn Fn` per file.

pub mod VStr;
pub mod ArgStr;
pub mod ArgString;
pub mod ArgStringOr;
pub mod ArgVal;
pub mod ArgU64;
pub mod ArgU64Or;
pub mod ArgI64;
pub mod ArgF64;
pub mod ArgBool;
pub mod ArgBoolTrue;
pub mod ReqStr;
pub mod ReqString;

// Re-exports for ergonomic import by callers
pub use VStr::Fn as Fn;
pub use ArgStr::Fn as ArgStr;
pub use ArgString::Fn as ArgString;
pub use ArgStringOr::Fn as ArgStringOr;
pub use ArgVal::Fn as ArgVal;
pub use ArgU64::Fn as ArgU64;
pub use ArgU64Or::Fn as ArgU64Or;
pub use ArgI64::Fn as ArgI64;
pub use ArgF64::Fn as ArgF64;
pub use ArgBool::Fn as ArgBool;
pub use ArgBoolTrue::Fn as ArgBoolTrue;
pub use ReqStr::Fn as ReqStr;
pub use ReqString::Fn as ReqString;
