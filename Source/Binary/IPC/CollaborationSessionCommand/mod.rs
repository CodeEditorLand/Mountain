//! `CollaborationSessionCommand` - atomized.

pub mod MountainCreateCollaborationSession;
pub mod MountainGetCollaborationSessions;

pub use MountainCreateCollaborationSession::Fn as MountainCreateCollaborationSession;
pub use MountainGetCollaborationSessions::Fn as MountainGetCollaborationSessions;
