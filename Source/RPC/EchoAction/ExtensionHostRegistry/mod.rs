pub mod New;
pub mod Record;
pub mod Forget;
pub mod Resolve;
pub mod Count;

use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

pub struct Struct {
	Hosts:Arc<RwLock<HashMap<String, String>>>,
}
