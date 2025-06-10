// @module webview (Handlers)
// @description This module contains the core logic for managing webview
// instances, including their creation, lifecycle, and communication with the
// Sky frontend. It aggregates and exports the handler functions from its
// sub-modules.
//

#![allow(non_snake_case, non_camel_case_types)]

mod WebviewLogic;

pub use self::WebviewLogic::*;
