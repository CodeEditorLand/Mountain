#![allow(non_snake_case)]

use std::borrow::Cow;

use tauri::http::header::{CONTENT_TYPE, HeaderValue};

#[allow(dead_code)]
pub fn Fn() {
	env_logger::Builder::new()
		.filter_level(log::LevelFilter::Debug)
		.format(|Buffer, Record| {
			use std::io::Write;

			use colored::Colorize;

			writeln!(
				Buffer,
				"[{}] [{}]: {}",
				"Mountain".red(),
				match Record.level() {
					log::Level::Error => "ERROR".red(),

					log::Level::Warn => "WARN".yellow(),

					log::Level::Info => "INFO".green(),

					log::Level::Debug => "DEBUG".blue(),

					log::Level::Trace => "TRACE".magenta(),
				},
				Record.args()
			)
		})
		.try_init()
		.expect("Failed to initialize env_logger");

	tokio::runtime::Builder::new_multi_thread()
		.enable_all()
		.build()
		.expect("Failed to build Tokio runtime.")
		.block_on(async {
			let mut Builder = tauri::Builder::default();

			#[cfg(any(windows, target_os = "linux"))]
			{
				Builder = Builder.any_thread();
			}

			Builder
				.setup(|Tauri| {
					let mut Builder = tauri::WebviewWindowBuilder::new(
						Tauri,
						"Application",
						tauri::WebviewUrl::App(std::path::PathBuf::from("Application/index.html")),
					)
					.on_web_resource_request(
						|Request:tauri::http::Request<Vec<u8>>,
						 Response:&mut tauri::http::Response<Cow<'static, [u8]>>| {
							let URI = Request.uri();

							let Path = URI.path();

							let Query = URI.query().unwrap_or("");

							if Path.ends_with(".css") {
								let Skip = Query.contains("Skip=Intercept");

								if Skip {
									match HeaderValue::from_static("text/css") {
										Mime => {
											if Response
												.headers()
												.get(CONTENT_TYPE)
												.map(|Value| Value != &Mime)
												.unwrap_or(true)
											{
												Response.headers_mut().insert(CONTENT_TYPE, Mime);
											}
										},
									}
								} else {
									*Response.body_mut() = Cow::Owned(
										format!(
											r#"if (typeof window._LOAD_CSS_WORKER === 'function') {{
	window._LOAD_CSS_WORKER("{}");
}}

export default {{}};"#,
											Path
										)
										.into_bytes(),
									);

									let Header = Response.headers_mut();

									Header.insert(
										CONTENT_TYPE,
										HeaderValue::from_static("application/javascript; charset=utf-8"),
									);

									Header.remove(tauri::http::header::CONTENT_LENGTH);

									Header.remove(tauri::http::header::ETAG);

									Header.remove(tauri::http::header::LAST_MODIFIED);

									*Response.status_mut() = tauri::http::StatusCode::OK;
								}
							}
						},
					)
					.use_https_scheme(true);

					#[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
					{
						Builder = Builder.title("FIDDEE").maximized(true);
					}

					let Window = match Builder.build() {
						Ok(Return) => Return,

						Err(_Error) => {
							panic!("Window build failed: {:?}", _Error);
						},
					};

					#[cfg(all(debug_assertions, not(any(target_os = "android", target_os = "ios"))))]
					{
						Window.open_devtools();
					}

					Ok(())
				})
				.run(tauri::generate_context!())
				.expect("Error while running Tauri application");
		});
}
