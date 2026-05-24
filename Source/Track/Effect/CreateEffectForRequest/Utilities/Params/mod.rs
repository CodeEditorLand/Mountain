//! `Params` utility helpers - one `pub fn Fn` per file.

pub mod StrAt;
pub mod StringAt;
pub mod StringAtOr;
pub mod ValAt;
pub mod U64At;
pub mod U64AtOr;
pub mod I64At;
pub mod I64AtOr;
pub mod BoolAt;
pub mod BoolAtTrue;
pub mod StrObjOrPos;
pub mod ObjStr;
pub mod ObjVal;
pub mod ObjBool;
pub mod ObjF64;
pub mod ArrayUnwrap;
pub mod UriFromParams;
pub mod EnsureArray;
pub mod StripFileUri;

// Re-exports for ergonomic import by callers
pub use StrAt::Fn as StrAt;
pub use StringAt::Fn as StringAt;
pub use StringAtOr::Fn as StringAtOr;
pub use ValAt::Fn as ValAt;
pub use U64At::Fn as U64At;
pub use U64AtOr::Fn as U64AtOr;
pub use I64At::Fn as I64At;
pub use I64AtOr::Fn as I64AtOr;
pub use BoolAt::Fn as BoolAt;
pub use BoolAtTrue::Fn as BoolAtTrue;
pub use StrObjOrPos::Fn as StrObjOrPos;
pub use ObjStr::Fn as ObjStr;
pub use ObjVal::Fn as ObjVal;
pub use ObjBool::Fn as ObjBool;
pub use ObjF64::Fn as ObjF64;
pub use ArrayUnwrap::Fn as ArrayUnwrap;
pub use UriFromParams::Fn as UriFromParams;
pub use EnsureArray::Fn as EnsureArray;
pub use StripFileUri::Fn as StripFileUri;
