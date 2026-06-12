//! Window-domain handlers for `CocoonService`. Sixteen entry points cover
//! show/hide messages, status-bar items, webview panels, and the prompt
//! family (quick-pick / input-box / progress).
/// CreateStatusBarItem handler: creates a status-bar item for an extension.
pub mod CreateStatusBarItem;

/// CreateWebviewPanel handler: creates a new webview panel.
pub mod CreateWebviewPanel;

/// DisposeWebviewPanel handler: disposes an existing webview panel.
pub mod DisposeWebviewPanel;

/// OnDidReceiveMessage handler: processes a message received from a webview.
pub mod OnDidReceiveMessage;

/// OpenExternal handler: opens a URI in an external application.
pub mod OpenExternal;

/// PostWebviewMessage handler: posts a message to a webview panel.
pub mod PostWebviewMessage;

/// ReportProgress handler: reports progress on a long-running operation.
pub mod ReportProgress;

/// SetStatusBarText handler: updates the text of a status-bar item.
pub mod SetStatusBarText;

/// SetWebviewHtml handler: sets the HTML content of a webview panel.
pub mod SetWebviewHtml;

/// ShowErrorMessage handler: displays an error message dialog to the user.
pub mod ShowErrorMessage;

/// ShowInformationMessage handler: displays an informational message dialog.
pub mod ShowInformationMessage;

/// ShowInputBox handler: prompts the user for text input.
pub mod ShowInputBox;

/// ShowProgress handler: displays a progress indicator in the UI.
pub mod ShowProgress;

/// ShowQuickPick handler: presents a quick-pick selection list to the user.
pub mod ShowQuickPick;

/// ShowTextDocument handler: opens and reveals a text document in the editor.
pub mod ShowTextDocument;

/// ShowWarningMessage handler: displays a warning message dialog to the user.
pub mod ShowWarningMessage;
