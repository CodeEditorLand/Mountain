//! Git command dispatcher.

use crate::Git::{
    HandleCancel::Fn as GitHandleCancel,
    HandleCheckout::Fn as GitHandleCheckout,
    HandleClone::Fn as GitHandleClone,
    HandleExec::Fn as GitHandleExec,
    HandleFetch::Fn as GitHandleFetch,
    HandleIsAvailable::Fn as GitHandleIsAvailable,
    HandlePull::Fn as GitHandlePull,
    HandleRevListCount::Fn as GitHandleRevListCount,
    HandleRevParse::Fn as GitHandleRevParse,
};

use serde_json::Value;

/// Dispatches git commands.
///
/// Handled commands:
/// - `git:exec`
/// - `git:clone`
/// - `git:pull`
/// - `git:checkout`
/// - `git:revParse`
/// - `git:fetch`
/// - `git:revListCount`
/// - `git:cancel`
/// - `git:isAvailable`
pub async fn dispatch_git(
    command: &str,

    arguments: Vec<Value>,
) -> Result<Value, String> {

    match command {
        "git:exec" => {
            crate::dev_log!("git", "git:exec");

            GitHandleExec(arguments).await
        },

        "git:clone" => {
            crate::dev_log!("git", "git:clone");

            GitHandleClone(arguments).await
        },

        "git:pull" => {
            crate::dev_log!("git", "git:pull");

            GitHandlePull(arguments).await
        },

        "git:checkout" => {
            crate::dev_log!("git", "git:checkout");

            GitHandleCheckout(arguments).await
        },

        "git:revParse" => {
            crate::dev_log!("git", "git:revParse");

            GitHandleRevParse(arguments).await
        },

        "git:fetch" => {
            crate::dev_log!("git", "git:fetch");

            GitHandleFetch(arguments).await
        },

        "git:revListCount" => {
            crate::dev_log!("git", "git:revListCount");

            GitHandleRevListCount(arguments).await
        },

        "git:cancel" => {
            crate::dev_log!("git", "git:cancel");

            GitHandleCancel(arguments).await
        },

        "git:isAvailable" => {
            crate::dev_log!("git", "git:isAvailable");

            GitHandleIsAvailable(arguments).await
        },

        _ => Err(format!("Unknown git command: {}", command)),
    }
}
