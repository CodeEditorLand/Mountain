#![allow(non_snake_case)]

#[allow(dead_code)]
pub fn Fn() {
	tokio::runtime::Builder::new_multi_thread()
		.enable_all()
		.build()
		.expect("Cannot build.")
		.block_on(async {
			let Builder = if cfg!(debug_assertions) {
				tauri::Builder::default().plugin(tauri_plugin_devtools::init())
			} else {
				tauri::Builder::default()
			};

			Builder
				.plugin(tauri_plugin_shell::init())
				// TODO: FIX THIS
				// .plugin(tauri_plugin_updater::Builder::new().build())
				.run(tauri::generate_context!())
				.expect("Cannot run.");
		});
}
