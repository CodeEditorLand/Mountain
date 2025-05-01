#![allow(non_snake_case)]

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
