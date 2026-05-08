#![allow(non_snake_case)]

//! Poll `IsClientConnected` every 50 ms until the sidecar appears in
//! the pool or the budget runs out. `BudgetMilliseconds` is a soft upper
//! bound; user-facing call paths should keep it under ~1500 ms.

use std::time::{Duration, Instant};

use crate::Vine::Client::{IsClientConnected, IsShuttingDown};

pub async fn Fn(SideCarIdentifier:&str, BudgetMilliseconds:u64) -> bool {
	if IsClientConnected::Fn(SideCarIdentifier) {
		return true;
	}

	let Deadline = Instant::now() + Duration::from_millis(BudgetMilliseconds);

	while Instant::now() < Deadline {
		tokio::time::sleep(Duration::from_millis(50)).await;

		if IsClientConnected::Fn(SideCarIdentifier) {
			return true;
		}

		if IsShuttingDown::Fn() {
			return false;
		}
	}

	IsClientConnected::Fn(SideCarIdentifier)
}
