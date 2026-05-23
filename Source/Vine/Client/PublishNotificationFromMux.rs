
//! Public-crate alias for `PublishNotification::Fn` so `Vine::Multiplexer`
//! can fan out notifications received over the streaming channel through
//! the same broadcast subscribers consume from.

use serde_json::Value;

use crate::Vine::Client::PublishNotification;

pub(crate) fn Fn(SideCarIdentifier:&str, Method:&str, Parameters:&Value) {
	PublishNotification::Fn(SideCarIdentifier, Method, Parameters);
}
