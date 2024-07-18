/// Represents a site that holds a WebSocket connection.
///
/// The `Site` struct contains an `Order` field, which is an `Arc` wrapped `Mutex`
/// that protects a WebSocket stream. This allows the WebSocket connection to be
/// safely shared and accessed across multiple asynchronous tasks.
///
pub struct Struct {
	/// The WebSocket connection wrapped in an `Arc` and `Mutex` for safe concurrent access.
	Order: Arc<
		Mutex<
			tokio_tungstenite::WebSocketStream<
				tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
			>,
		>,
	>,
}

/// Implements the `Worker` trait for the `Site` struct, allowing it to process actions.
///
/// This implementation sends actions over a WebSocket connection and waits for a response.
/// The response is then deserialized and returned as an `ActionResult`.
///
/// # Arguments
///
/// * `Action` - The action to be processed.
///
/// # Returns
///
/// An `ActionResult` containing the result of the action.
///
/// # Errors
///
/// Returns an `ActionResult` with an error message if:
/// - Sending the action over the WebSocket connection fails.
/// - Receiving a response from the WebSocket connection fails.
/// - Deserializing the response fails.
///
#[async_trait::async_trait]
impl Echo::Fn::Job::Worker for Struct {
	async fn Receive(&self, Action: Action) -> ActionResult {
		let mut Order = self.Order.lock().await;

		if Order.send(Text(serde_json::to_string(&Action).unwrap())).await.is_err() {
			return ActionResult { Action, Result: Err("Cannot ActionResult.".to_string()) };
		}

		if let Some(response) = Order.next().await {
			match response {
				Ok(Text(text)) => serde_json::from_str(&text).unwrap_or(ActionResult {
					Action,
					Result: Err("Cannot serde_json.".to_string()),
				}),
				_ => ActionResult { Action, Result: Err("Cannot ActionResult.".to_string()) },
			}
		} else {
			ActionResult { Action, Result: Err("Cannot ActionResult.".to_string()) }
		}
	}
}
