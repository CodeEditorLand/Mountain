//! Capture a named event with optional properties. Stamps the standard
//! Mountain identity (`$app`, `$app_version`, `$build_mode`,
//! `$component`) on every event before merging caller props.

use crate::Binary::Build::PostHogPlugin::{CaptureAllowed, Client, DistinctId};

pub fn Fn(EventName:&str, Properties:Option<Vec<(&str, &str)>>) {

	if !CaptureAllowed::Fn() {
		return;
	}

	let Some(C) = Client::CLIENT.get() else { return };

	let mut Event = posthog_rs::Event::new(EventName, &DistinctId::Fn());

	let _ = Event.insert_prop("$app", "fiddee");

	let _ = Event.insert_prop("$app_version", "0.0.1");

	let _ = Event.insert_prop("$build_mode", "debug");

	let _ = Event.insert_prop("$component", "mountain");

	if let Some(Props) = Properties {
		for (Key, Value) in Props {
			let _ = Event.insert_prop(Key, Value);
		}
	}

	let _ = C.capture(Event);
}
