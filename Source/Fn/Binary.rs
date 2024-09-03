#![allow(non_snake_case)]

#[allow(dead_code)]
pub fn Fn() {
	tokio::runtime::Builder::new_multi_thread()
		.enable_all()
		.build()
		.expect("Cannot new_multi_thread.")
		.block_on(async {
			let Builder = tauri::Builder::default();

			// TODO: FIX THIS
			// #[cfg(debug_assertions)]
			// Builder.plugin(tauri_plugin_devtools::init());

			Builder
				.plugin(tauri_plugin_shell::init())
				.plugin(tauri_plugin_updater::Builder::new().build())
				.run(tauri::generate_context!())
				.expect("Cannot Library.");
		});
}
