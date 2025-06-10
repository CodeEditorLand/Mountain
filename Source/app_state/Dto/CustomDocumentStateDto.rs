

/**
 * @module CustomDocumentStateDto
 * @description Defines the Data Transfer Object for storing the state of a single
 * custom editor document.
 */

#![allow(non_snake_case, non_camel_case_types)]

use super::super::Internal::UrlSerdeHelper;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use url::Url;

/**
 * A struct that holds the state for a document being handled by a custom editor.
 * This is stored in `AppState` to track the lifecycle of custom documents.
 */
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct CustomDocumentStateDto {
    /// The URI of the document resource being edited.
    #[serde(with = "UrlSerdeHelper")]
    pub Uri: Url,

    /// The view type of the custom editor responsible for this document.
    pub ViewType: String,

    /// The identifier of the sidecar process where the custom editor provider lives.
    pub SidecarIdentifier: String,

    /// A flag indicating if the document is currently editable by the user.
    pub IsEditable: bool,

    /// An optional identifier for a backup copy of the file's content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub BackupId: Option<String>,

    /// A map to store edit history or other versioning information.
    /// In a real implementation, this might hold a more structured edit type.
    pub Edits: HashMap<u32, serde_json::Value>,
}
