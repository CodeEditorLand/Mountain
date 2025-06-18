// @module sky_user_interface_response (Handler)
// @description This module contains the logic for handling asynchronous
// responses sent from the Sky frontend back to the Mountain backend. This is
// the receiving end of the request-response pattern used for UI interactions
// like dialogs. Renamed from `SkyUserInterfaceResponse`.
//

#![allow(non_snake_case)]

mod SkyUserInterfaceResponseLogic;

pub use self::SkyUserInterfaceResponseLogic::*;
