
//! # Navigation history + URI labels
//!
//! Two related responsibilities sharing the same dispatcher
//! family:
//!
//! - `History*` - back/forward chain over editor URIs. Drives the workbench's
//!   back/forward buttons + the Cmd+Alt+- jump list. Stack lives in
//!   `ApplicationState.Feature.NavigationHistory`.
//! - `Label*` - URI → human-readable label resolution. Used by tabs,
//!   breadcrumbs, and quick-open labels.
//!
//! Layout (one export per file, file name = identity):
//! - `HistoryGoBack::HistoryGoBack`, `HistoryGoForward::HistoryGoForward`,
//!   `HistoryCanGoBack::HistoryCanGoBack`,
//!   `HistoryCanGoForward::HistoryCanGoForward`, `HistoryPush::HistoryPush`,
//!   `HistoryClear::HistoryClear`, `HistoryGetStack::HistoryGetStack`.
//! - `LabelGetURI::LabelGetURI`, `LabelGetWorkspace::LabelGetWorkspace`,
//!   `LabelGetBase::LabelGetBase`.

pub mod HistoryCanGoBack;

pub mod HistoryCanGoForward;

pub mod HistoryClear;

pub mod HistoryGetStack;

pub mod HistoryGoBack;

pub mod HistoryGoForward;

pub mod HistoryPush;

pub mod LabelGetBase;

pub mod LabelGetURI;

pub mod LabelGetWorkspace;
