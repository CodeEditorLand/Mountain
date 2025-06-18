// @module webview (Handler)
// @description This module contains the core logic for managing webview
// instances, including their creation, lifecycle, and communication with the
// Sky frontend. It aggregates and exports the handler functions from its
// sub-modules. Renamed from `WebView`.
//

#![allow(non_snake_case)]

mod WebViewLogic;

pub use self::WebViewLogic::*;
