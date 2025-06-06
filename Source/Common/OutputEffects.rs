// File: Common/OutputEffect.rs
// Defines the OutputChannelManager trait and associated effects for managing
// output channels, which are used to display textual information to the user.

#![allow(non_snake_case, non_camel_case_types)]

use std::sync::Arc;

use async_trait::async_trait;

use crate::{
	Effect::ActionEffect,
	Environment::{Environment, Requires},
	Errors::CommonError,
	Runtime::AppRuntimeTrait,
};

/// A trait for environments that can manage output channels.
#[async_trait]
pub trait OutputChannelManager: Environment {
	/// Registers a new output channel.
	async fn RegisterChannel(&self, Name:String, LanguageIdentifier:Option<String>) -> Result<String, CommonError>;
	/// Appends a string to an existing channel.
	async fn Append(&self, ChannelIdentifier:String, Value:String) -> Result<(), CommonError>;
	/// Replaces the entire content of a channel with a new string.
	async fn Replace(&self, ChannelIdentifier:String, Value:String) -> Result<(), CommonError>;
	/// Clears all content from a channel.
	async fn Clear(&self, ChannelIdentifier:String) -> Result<(), CommonError>;
	/// Makes a channel visible in the UI.
	async fn Reveal(&self, ChannelIdentifier:String, PreserveFocus:bool) -> Result<(), CommonError>;
	/// Hides a channel's view in the UI.
	async fn Close(&self, ChannelIdentifier:String) -> Result<(), CommonError>;
	/// Completely removes and disposes of a channel.
	async fn Dispose(&self, ChannelIdentifier:String) -> Result<(), CommonError>;
}

/// Creates an effect to register a new output channel.
pub fn RegisterOutputChannel<RuntimeAccessType>(
	Name:String,
	LanguageIdentifier:Option<String>,
) -> ActionEffect<Arc<RuntimeAccessType>, CommonError, String>
where
	RuntimeAccessType: AppRuntimeTrait<RuntimeAccessType::EnvironmentType> + Send + Sync + 'static,
	RuntimeAccessType::EnvironmentType: Requires<Arc<dyn OutputChannelManager>>, {
	ActionEffect::New(Arc::new(move |Accessor:Arc<RuntimeAccessType>| {
		let NameClone = Name.clone();
		let LanguageIdentifierClone = LanguageIdentifier.clone();
		Box::pin(async move {
			let Environment = Accessor.GetEnvironment();
			let Manager:Arc<dyn OutputChannelManager> = Environment.require();
			Manager.RegisterChannel(NameClone, LanguageIdentifierClone).await
		})
	}))
}

/// Creates an effect to append text to an output channel.
pub fn AppendToOutputChannel<RuntimeAccessType>(
	ChannelIdentifier:String,
	Value:String,
) -> ActionEffect<Arc<RuntimeAccessType>, CommonError, ()>
where
	RuntimeAccessType: AppRuntimeTrait<RuntimeAccessType::EnvironmentType> + Send + Sync + 'static,
	RuntimeAccessType::EnvironmentType: Requires<Arc<dyn OutputChannelManager>>, {
	ActionEffect::New(Arc::new(move |Accessor:Arc<RuntimeAccessType>| {
		let ChannelIdentifierClone = ChannelIdentifier.clone();
		let ValueClone = Value.clone();
		Box::pin(async move {
			let Environment = Accessor.GetEnvironment();
			let Manager:Arc<dyn OutputChannelManager> = Environment.require();
			Manager.Append(ChannelIdentifierClone, ValueClone).await
		})
	}))
}

/// Creates an effect to replace the content of an output channel.
pub fn ReplaceOutputChannelContent<RuntimeAccessType>(
	ChannelIdentifier:String,
	Value:String,
) -> ActionEffect<Arc<RuntimeAccessType>, CommonError, ()>
where
	RuntimeAccessType: AppRuntimeTrait<RuntimeAccessType::EnvironmentType> + Send + Sync + 'static,
	RuntimeAccessType::EnvironmentType: Requires<Arc<dyn OutputChannelManager>>, {
	ActionEffect::New(Arc::new(move |Accessor:Arc<RuntimeAccessType>| {
		let ChannelIdentifierClone = ChannelIdentifier.clone();
		let ValueClone = Value.clone();
		Box::pin(async move {
			let Environment = Accessor.GetEnvironment();
			let Manager:Arc<dyn OutputChannelManager> = Environment.require();
			Manager.Replace(ChannelIdentifierClone, ValueClone).await
		})
	}))
}

/// Creates an effect to clear an output channel.
pub fn ClearOutputChannel<RuntimeAccessType>(
	ChannelIdentifier:String,
) -> ActionEffect<Arc<RuntimeAccessType>, CommonError, ()>
where
	RuntimeAccessType: AppRuntimeTrait<RuntimeAccessType::EnvironmentType> + Send + Sync + 'static,
	RuntimeAccessType::EnvironmentType: Requires<Arc<dyn OutputChannelManager>>, {
	ActionEffect::New(Arc::new(move |Accessor:Arc<RuntimeAccessType>| {
		let ChannelIdentifierClone = ChannelIdentifier.clone();
		Box::pin(async move {
			let Environment = Accessor.GetEnvironment();
			let Manager:Arc<dyn OutputChannelManager> = Environment.require();
			Manager.Clear(ChannelIdentifierClone).await
		})
	}))
}

/// Creates an effect to reveal an output channel.
pub fn RevealOutputChannel<RuntimeAccessType>(
	ChannelIdentifier:String,
	PreserveFocus:bool,
) -> ActionEffect<Arc<RuntimeAccessType>, CommonError, ()>
where
	RuntimeAccessType: AppRuntimeTrait<RuntimeAccessType::EnvironmentType> + Send + Sync + 'static,
	RuntimeAccessType::EnvironmentType: Requires<Arc<dyn OutputChannelManager>>, {
	ActionEffect::New(Arc::new(move |Accessor:Arc<RuntimeAccessType>| {
		let ChannelIdentifierClone = ChannelIdentifier.clone();
		Box::pin(async move {
			let Environment = Accessor.GetEnvironment();
			let Manager:Arc<dyn OutputChannelManager> = Environment.require();
			Manager.Reveal(ChannelIdentifierClone, PreserveFocus).await
		})
	}))
}

/// Creates an effect to close the view of an output channel.
pub fn CloseOutputChannelView<RuntimeAccessType>(
	ChannelIdentifier:String,
) -> ActionEffect<Arc<RuntimeAccessType>, CommonError, ()>
where
	RuntimeAccessType: AppRuntimeTrait<RuntimeAccessType::EnvironmentType> + Send + Sync + 'static,
	RuntimeAccessType::EnvironmentType: Requires<Arc<dyn OutputChannelManager>>, {
	ActionEffect::New(Arc::new(move |Accessor:Arc<RuntimeAccessType>| {
		let ChannelIdentifierClone = ChannelIdentifier.clone();
		Box::pin(async move {
			let Environment = Accessor.GetEnvironment();
			let Manager:Arc<dyn OutputChannelManager> = Environment.require();
			Manager.Close(ChannelIdentifierClone).await
		})
	}))
}

/// Creates an effect to dispose of an output channel.
pub fn DisposeOutputChannel<RuntimeAccessType>(
	ChannelIdentifier:String,
) -> ActionEffect<Arc<RuntimeAccessType>, CommonError, ()>
where
	RuntimeAccessType: AppRuntimeTrait<RuntimeAccessType::EnvironmentType> + Send + Sync + 'static,
	RuntimeAccessType::EnvironmentType: Requires<Arc<dyn OutputChannelManager>>, {
	ActionEffect::New(Arc::new(move |Accessor:Arc<RuntimeAccessType>| {
		let ChannelIdentifierClone = ChannelIdentifier.clone();
		Box::pin(async move {
			let Environment = Accessor.GetEnvironment();
			let Manager:Arc<dyn OutputChannelManager> = Environment.require();
			Manager.Dispose(ChannelIdentifierClone).await
		})
	}))
}
