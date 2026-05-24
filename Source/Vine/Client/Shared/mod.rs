//! `Shared` - atomized.

pub mod GetConnectionNotify;
pub mod FireConnectionNotify;
pub mod ShutdownFlagStore;
pub mod ShutdownFlagLoad;
pub mod RecordSideCarFailure;
pub mod UpdateSideCarActivity;
pub mod ValidateMessageSize;

pub use GetConnectionNotify::Fn as GetConnectionNotify;
pub use FireConnectionNotify::Fn as FireConnectionNotify;
pub use ShutdownFlagStore::Fn as ShutdownFlagStore;
pub use ShutdownFlagLoad::Fn as ShutdownFlagLoad;
pub use RecordSideCarFailure::Fn as RecordSideCarFailure;
pub use UpdateSideCarActivity::Fn as UpdateSideCarActivity;
pub use ValidateMessageSize::Fn as ValidateMessageSize;
