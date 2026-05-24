//! `VsixInstaller` - atomized.

pub mod InstallVsix;
pub mod UninstallExtension;
pub mod ReadFullManifest;
pub mod HealExecutableBits;

pub use InstallVsix::Fn as InstallVsix;
pub use UninstallExtension::Fn as UninstallExtension;
pub use ReadFullManifest::Fn as ReadFullManifest;
pub use HealExecutableBits::Fn as HealExecutableBits;
