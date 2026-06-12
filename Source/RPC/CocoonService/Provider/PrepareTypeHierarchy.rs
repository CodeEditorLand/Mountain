//! `PrepareTypeHierarchy` gRPC RPC handler.
//!
//! Entry point for VS Code's type hierarchy feature. Returns the root
//! `TypeHierarchyItem` at the given position so the Subtypes/Supertypes
//! panels have a starting item to display.

use serde_json::Value;
use tonic::{Response, Status};
use url::Url;
use CommonLibrary::LanguageFeature::{
	DTO::PositionDTO::PositionDTO,
	LanguageFeatureProviderRegistry::LanguageFeatureProviderRegistry,
};
use ::Vine::Generated::{
	Position,
	ProvideTypeHierarchyRequest,
	ProvideTypeHierarchyResponse,
	Range,
	TypeHierarchyItem,
	Uri,
};

use crate::{RPC::CocoonService::CocoonServiceImpl, dev_log};

/// Maps a serialized `vscode.Uri` (object with `external`/`scheme`/`path`
/// fields, or a plain string) into the proto `Uri`.
fn JsonToUri(UriValue:Option<&Value>) -> Option<Uri> {
	let V = UriValue?;

	if let Some(S) = V.as_str() {
		return Some(Uri { value:S.to_string() });
	}

	if let Some(External) = V.get("external").and_then(|E| E.as_str()) {
		return Some(Uri { value:External.to_string() });
	}

	let Scheme = V.get("scheme").and_then(|S| S.as_str()).unwrap_or("file");

	let Authority = V.get("authority").and_then(|A| A.as_str()).unwrap_or("");

	let Path = V.get("path").and_then(|P| P.as_str()).unwrap_or("");

	if Path.is_empty() {
		return None;
	}

	Some(Uri { value:format!("{}://{}{}", Scheme, Authority, Path) })
}

fn JsonToPosition(PositionValue:&Value) -> Position {
	Position {
		line:PositionValue
			.get("line")
			.or_else(|| PositionValue.get("Line"))
			.and_then(|L| L.as_u64())
			.unwrap_or(0) as u32,

		character:PositionValue
			.get("character")
			.or_else(|| PositionValue.get("Character"))
			.and_then(|C| C.as_u64())
			.unwrap_or(0) as u32,
	}
}

/// Maps a serialized `vscode.Range` into the proto `Range`. extHostTypes
/// `Range.toJSON()` emits a `[start, end]` pair; plain provider objects
/// use `{ start, end }`.
fn JsonToRange(RangeValue:Option<&Value>) -> Option<Range> {
	let V = RangeValue?;

	let (Start, End) = if let Some(Pair) = V.as_array() {
		(Pair.first()?, Pair.get(1)?)
	} else {
		(V.get("start")?, V.get("end")?)
	};

	Some(Range { start:Some(JsonToPosition(Start)), end:Some(JsonToPosition(End)) })
}

fn JsonToItem(ItemValue:&Value) -> TypeHierarchyItem {
	TypeHierarchyItem {
		name:ItemValue.get("name").and_then(|N| N.as_str()).unwrap_or("").to_string(),

		kind:ItemValue.get("kind").and_then(|K| K.as_u64()).unwrap_or(0) as u32,

		uri:JsonToUri(ItemValue.get("uri")),

		range:JsonToRange(ItemValue.get("range")),

		selection_range:JsonToRange(ItemValue.get("selectionRange")),

		detail:ItemValue.get("detail").and_then(|D| D.as_str()).unwrap_or("").to_string(),
	}
}

pub async fn Fn(
	Service:&CocoonServiceImpl,

	Request:ProvideTypeHierarchyRequest,
) -> Result<Response<ProvideTypeHierarchyResponse>, Status> {
	let URI = Request.uri.as_ref().map(|U| U.value.as_str()).unwrap_or("");

	let Position_ = Request.position.as_ref();

	let Line = Position_.map(|P| P.line).unwrap_or(0);

	let Character = Position_.map(|P| P.character).unwrap_or(0);

	dev_log!(
		"provider",
		"PrepareTypeHierarchy handle={} uri={} line={} char={}",
		Request.provider_handle,
		URI,
		Line,
		Character
	);

	let DocumentURI = Url::parse(URI).map_err(|E| Status::invalid_argument(format!("Invalid URI: {}", E)))?;

	let PositionDTO_ = PositionDTO { LineNumber:Line, Column:Character };

	let Forward = Service.environment.PrepareTypeHierarchy(DocumentURI, PositionDTO_);

	let Outcome = match Service.RunCancellable("PrepareTypeHierarchy", Forward).await {
		Some(Outcome) => Outcome,

		None => return Ok(Response::new(ProvideTypeHierarchyResponse::default())),
	};

	match Outcome {
		Ok(Some(Items)) => {
			let Mapped:Vec<TypeHierarchyItem> = Items
				.as_array()
				.map(|A| A.as_slice())
				.unwrap_or_else(|| std::slice::from_ref(&Items))
				.iter()
				.filter(|I| I.is_object())
				.map(JsonToItem)
				.collect();

			dev_log!(
				"provider",
				"PrepareTypeHierarchy result handle={} items={}",
				Request.provider_handle,
				Mapped.len()
			);

			Ok(Response::new(ProvideTypeHierarchyResponse { items:Mapped }))
		},

		Ok(None) => Ok(Response::new(ProvideTypeHierarchyResponse::default())),

		Err(Error) => Err(Status::internal(format!("prepare type hierarchy failed: {}", Error))),
	}
}
