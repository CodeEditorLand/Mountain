#![allow(non_snake_case)]

//! Spawn a tokio task that logs cache stats every 30 s under the `path-canon`
//! trace tag. Optional; call from `RunTime::Setup` when the user has
//! `Trace=path-canon` enabled.

use std::time::Duration;

use crate::{Cache::PathCanon::Stats, dev_log};

pub fn Fn() {
	tokio::spawn(async {
		let mut Interval = tokio::time::interval(Duration::from_secs(30));

		// skip the immediate first tick
		Interval.tick().await;

		loop {
			Interval.tick().await;

			let Snapshot = Stats::Fn();

			dev_log!(
				"path-canon",
				"entries={} weighted={}",
				Snapshot.Entries,
				Snapshot.WeightedSize
			);
		}
	});
}
