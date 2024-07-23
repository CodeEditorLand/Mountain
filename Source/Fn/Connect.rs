/// Asynchronously establishes a WebSocket connection to a specified URL with retry logic.
///
/// This function attempts to connect to a WebSocket server at the specified URL. If the connection attempt fails,
/// it retries a specified number of times with a delay between attempts.
///
/// # Returns
/// A `WebSocketStream` representing the WebSocket connection stream.
///
/// # Panics
/// Panics if the maximum number of retries is reached without a successful connection.
///
/// # Examples
/// ```
/// use tokio_tungstenite::WebSocketStream;
/// use tokio_tungstenite::MaybeTlsStream;
/// use tokio::net::TcpStream;
/// use std::time::Duration;
///
/// let stream = Fn().await;
/// ```
pub async fn Fn(
) -> tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>> {
	let mut Current = 0;
	let Again = 5;
	let Wait = std::time::Duration::from_secs(5);

	loop {
		match tokio_tungstenite::connect_async("ws://localhost:9999").await {
			Ok((Stream, _)) => return Stream,
			Err(e) => {
				if Current >= Again {
					panic!("Cannot {} retries: {}", Again, e);
				}

				println!("Connection attempt failed: {}. Retrying in {:?}...", e, Wait);

				tokio::time::sleep(Wait).await;

				Current += 1;
			}
		}
	}
}
