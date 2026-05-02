#![allow(non_snake_case)]

//! Clear the entire asset cache. Called on shutdown or on an explicit flush
//! signal.

use crate::Cache::AssetMemoryMap::Map;

pub fn Fn() { Map::Fn().clear(); }
