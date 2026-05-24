//! `Scheme` - atomized.

pub mod InitServiceRegistry;
pub mod LandSchemeHandler;
pub mod RegisterLandService;
pub mod GetLandPort;
pub mod LandSchemeHandlerAsync;
pub mod Scheme;
pub mod VscodeFileSchemeHandler;
pub mod VscodeWebviewSchemeHandler;

pub use InitServiceRegistry::Fn as InitServiceRegistry;
pub use LandSchemeHandler::Fn as LandSchemeHandler;
pub use RegisterLandService::Fn as RegisterLandService;
pub use GetLandPort::Fn as GetLandPort;
pub use LandSchemeHandlerAsync::Fn as LandSchemeHandlerAsync;
pub use Scheme::Fn as Scheme;
pub use VscodeFileSchemeHandler::Fn as VscodeFileSchemeHandler;
pub use VscodeWebviewSchemeHandler::Fn as VscodeWebviewSchemeHandler;
