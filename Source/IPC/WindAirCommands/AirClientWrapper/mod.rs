pub mod New;
pub mod Reconnect;

use crate::{Air::Struct as AirClientModule, dev_log};

#[derive(Debug, Clone)]
pub struct Struct {
	pub(super) client:AirClientModule::AirClient,
}
