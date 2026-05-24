//! `VineSubscribeCommand::VineSubscriberCount`

use serde::Serialize;
use serde_json::Value;
use tauri::ipc::Channel;
use crate::{Vine::Client::SubscribeNotifications::Fn as SubscribeNotifications, dev_log};

/// Diagnostic: how many active subscribers exist on the broadcast.
/// Useful from the frontend for verifying that prior subscriptions
/// haven't leaked across reloads.
#[tauri::command]
pub async fn Fn() -> Result<usize, String> { Ok(crate::Vine::Client::SubscriberCount::Fn()) }
