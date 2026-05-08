//! Provider lookup and matching utilities.

use CommonLibrary::{Error::CommonError::CommonError, LanguageFeature::DTO::ProviderType::ProviderType};
use url::Url;

use crate::{
	ApplicationState::DTO::ProviderRegistrationDTO::ProviderRegistrationDTO,
	Environment::Utility::ErrorMapping::MapApplicationStateLockErrorToCommonError,
	dev_log,
};

pub(super) async fn get_matching_provider(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	document_uri:&Url,

	feature_type:ProviderType,
) -> Result<Option<ProviderRegistrationDTO>, CommonError> {
	let providers = environment
		.ApplicationState
		.Extension
		.ProviderRegistration
		.LanguageProviders
		.lock()
		.map_err(MapApplicationStateLockErrorToCommonError)?;

	let open_documents = environment
		.ApplicationState
		.Feature
		.Documents
		.OpenDocuments
		.lock()
		.map_err(MapApplicationStateLockErrorToCommonError)?;

	// Derive language: prefer DocumentState record, fall back to URI extension.
	let LanguageId:String = if let Some(Document) = open_documents.get(document_uri.as_str()) {
		Document.LanguageIdentifier.clone()
	} else {
		// Document not yet opened via model:open - infer from file extension.
		document_uri
			.path()
			.split('.')
			.next_back()
			.map(|Ext| {
				match Ext {
					"rs" => "rust",
					"ts" | "tsx" => "typescript",
					"js" | "jsx" | "mjs" | "cjs" => "javascript",
					"json" | "jsonc" => "json",
					"toml" => "toml",
					"yaml" | "yml" => "yaml",
					"md" => "markdown",
					"py" => "python",
					"go" => "go",
					"c" | "h" => "c",
					"cpp" | "cc" | "cxx" | "hpp" => "cpp",
					Other => Other,
				}
			})
			.unwrap_or("plaintext")
			.to_string()
	};

	for Provider in providers.values() {
		if Provider.ProviderType != feature_type {
			continue;
		}

		// Selector shapes (all stored as JSON from CocoonService.RegisterProvider):
		//   Canonical: [{ "language": "typescript" }]
		//   Wildcard:  [{ "language": "*" }]
		//   Legacy obj: { "language": ["typescript"] }
		//   Plain str: "*"
		let Matched = if let Some(SelectorArray) = Provider.Selector.as_array() {
			SelectorArray.iter().any(|S| {
				match S.get("language") {
					Some(L) if L.as_str() == Some(&LanguageId) => true,
					Some(L) if L.as_str() == Some("*") => true,
					Some(L) => {
						L.as_array()
							.map(|Arr| {
								Arr.iter()
									.any(|Item| Item.as_str() == Some(&LanguageId) || Item.as_str() == Some("*"))
							})
							.unwrap_or(false)
					},
					None => false,
				}
			})
		} else if let Some(LangValue) = Provider.Selector.get("language") {
			LangValue.as_str() == Some(&LanguageId)
				|| LangValue.as_str() == Some("*")
				|| LangValue
					.as_array()
					.map(|Arr| {
						Arr.iter()
							.any(|Item| Item.as_str() == Some(&LanguageId) || Item.as_str() == Some("*"))
					})
					.unwrap_or(false)
		} else if let Some(LangStr) = Provider.Selector.as_str() {
			LangStr == &LanguageId || LangStr == "*"
		} else {
			false
		};

		if Matched {
			return Ok(Some(Provider.clone()));
		}
	}

	dev_log!(
		"extensions",
		"warn: [ProviderLookup] No {:?} provider for language '{}' (uri={})",
		feature_type,
		LanguageId,
		document_uri
	);

	Ok(None)
}
