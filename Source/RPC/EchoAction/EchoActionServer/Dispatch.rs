//! `EchoActionServer::Dispatch`

use super::Struct;
use std::sync::Arc;
use Echo::{Scheduler::Scheduler::Scheduler, Task::Priority::Priority as EchoPriority};
use tokio::sync::oneshot;
use crate::RPC::EchoAction::{ExtensionHostRegistry, ResolveMethodPriority};

pub fn Fn<F, T>(&self, Scheduler:&Scheduler, Method:&str, Task:F) -> Result<T, String>
	where
		F: std::future::Future<Output = T> + Send + 'static,
		T: Send + 'static, {
		let Priority = ResolveMethodPriority::Fn(Method);

		let (Sender, Receiver) = oneshot::channel::<T>();

		Scheduler.Submit(
			async move {
				let Output = Task.await;
				let _ = Sender.send(Output);
			},
			Priority,
		);

		Receiver
			.await
			.map_err(|_| "EchoAction task cancelled before completion".to_string())
	}
