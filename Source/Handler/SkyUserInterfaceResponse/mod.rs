

/**
 * @module sky_ui_responses (Handlers)
 * @description This module contains the logic for handling asynchronous responses
 * sent from the Sky frontend back to the Mountain backend. This is the receiving
 * end of the request-response pattern used for UI interactions like dialogs.
 */

#![allow(non_snake_case, non_camel_case_types)]

mod SkyUiResponsesLogic;

pub use self::SkyUiResponsesLogic::*;
