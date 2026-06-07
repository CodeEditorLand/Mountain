fn UpdateScmGroupMarkers(&self, ScmHandle:u32, GroupId:&str, ResourceStates:&Value) {

	use std::collections::HashMap;

	use CommonLibrary::SourceControlManagement::DTO::SourceControlManagementResourceDTO::SourceControlManagementResourceDTO;

	let mut Resources = self
		.RunTime()
		.Environment
		.ApplicationState
		.Feature
		.Markers
		.SourceControlManagementResources
		.lock();

	let GroupsForProvider = Resources.entry(ScmHandle).or_insert_with(HashMap::new);

	let mut DtoList:Vec<SourceControlManagementResourceDTO> = Vec::new();

	if let Some(Array) = ResourceStates.as_array() {
		for Raw in Array {
			let ResourceUri = Raw
				.get("resourceUri")
				.or_else(|| Raw.get("sourceUri"))
				.or_else(|| Raw.get("uri"))
				.cloned()
				.unwrap_or(Value::Null);

			if ResourceUri.is_null() {
				continue;
			}

			let Decorations = Raw
				.get("decorations")
				.cloned()
				.unwrap_or_else(|| Value::Object(serde_json::Map::new()));

			DtoList.push(SourceControlManagementResourceDTO {
				ProviderHandle:ScmHandle,
				GroupIdentifier:GroupId.to_string(),
				ResourceURI:ResourceUri,
				Decorations,
			});
		}
	}

	GroupsForProvider.insert(GroupId.to_string(), DtoList);
}
