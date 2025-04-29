#![allow(non_snake_case)]

#[allow(dead_code)]
pub fn Fn() {
	tokio::runtime::Builder::new_multi_thread()
		.enable_all()
		.build()
		.expect("Cannot build.")
		.block_on(async {
			let mut Builder = tauri::Builder::default();

			#[cfg(any(windows, target_os = "linux"))]
			{
				Builder = Builder.any_thread();
			}

			Builder
				.setup(|Tauri| {
					let mut Application = tauri::WebviewWindowBuilder::new(
						Tauri,
						"Application",
						tauri::WebviewUrl::App(std::path::PathBuf::from("Application")),
					);

					#[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
					{
						Application = Application.title("FIDDEE").maximized(true).theme(Some(tauri::Theme::Light));
					}

					let Window = Application.build().expect("Cannot build.");

					#[cfg(debug_assertions)]
					{
						Window.open_devtools();
					}

					Ok(())
				})
				.run(tauri::generate_context!())
				.expect("Cannot run.");
		});
}
