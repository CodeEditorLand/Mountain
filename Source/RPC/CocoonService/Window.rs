//! Window-domain handlers for `CocoonService`. Sixteen entry points cover
//! show/hide messages, status-bar items, webview panels, and the prompt
//! family (quick-pick / input-box / progress).
pub mod CreateStatusBarItem;

pub mod CreateWebviewPanel;

pub mod DisposeWebviewPanel;

pub mod OnDidReceiveMessage;

pub mod OpenExternal;

pub mod PostWebviewMessage;

pub mod ReportProgress;

pub mod SetStatusBarText;

pub mod SetWebviewHtml;

pub mod ShowErrorMessage;

pub mod ShowInformationMessage;

pub mod ShowInputBox;

pub mod ShowProgress;

pub mod ShowQuickPick;

pub mod ShowTextDocument;

pub mod ShowWarningMessage;
