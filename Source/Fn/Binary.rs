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
					let mut Daemon =
						tauri::WebviewWindowBuilder::new(Tauri, "Daemon", tauri::WebviewUrl::App("index.html".into()))
							.accept_first_mouse(false)
							.transparent(true)
							.user_agent("")
							.zoom_hotkeys_enabled(false);

					#[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
					{
						Daemon = Daemon
							.position(0.0, 0.0)
							.visible(true)
							.title("")
							.always_on_bottom(false)
							.closable(false)
							.decorations(false)
							.fullscreen(true)
							.theme(Some(tauri::Theme::Light));
					}

					Daemon.build().expect("Cannot build.");

					Ok(())
				})
				.run(tauri::generate_context!())
				.expect("Cannot run.");
		});
}
