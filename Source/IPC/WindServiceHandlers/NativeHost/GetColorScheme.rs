#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! Wire method: `nativeHost:getColorScheme`.
//! Returns `{ dark, highContrast }`. Dark-mode probe covers macOS
//! `AppleInterfaceStyle`, Windows `AppsUseLightTheme`, and a Linux ladder
//! (GTK color-scheme → GTK theme name → `KDE_COLOR_SCHEME` → xfconf).
//! High-contrast probe is Windows `HighContrast/Flags` and the GNOME a11y
//! `high-contrast` key; other OSes return false.

use std::sync::OnceLock;

use serde_json::{Value, json};

// Cache dark-mode result for the process lifetime. The system colour scheme
// is queried by spawning `defaults`/`reg`/`gsettings` which adds ~5-15 ms
// on cold start. The workbench calls `getOSColorScheme` during boot and again
// on window focus; caching turns the second+ call into a sub-microsecond read.
// If the user switches dark/light mode while editing, they are expected to
// restart the editor (same behaviour as stock VS Code on Electron).
static DARK_MODE_CACHE:OnceLock<bool> = OnceLock::new();

pub async fn NativeGetColorScheme() -> Result<Value, String> {
	let Dark = *DARK_MODE_CACHE.get_or_init(detect_dark_mode);

	let HighContrast = {
		#[cfg(target_os = "windows")]
		{
			std::process::Command::new("reg")
				.args(["query", "HKCU\\Control Panel\\Accessibility\\HighContrast", "/v", "Flags"])
				.output()
				.ok()
				.map(|O| {
					let Output = String::from_utf8_lossy(&O.stdout);
					Output.contains("0x1") || Output.contains("REG_DWORD    1")
				})
				.unwrap_or(false)
		}

		#[cfg(not(target_os = "windows"))]
		{
			#[cfg(target_os = "linux")]
			{
				std::process::Command::new("gsettings")
					.args(["get", "org.gnome.desktop.a11y.interface", "high-contrast"])
					.output()
					.ok()
					.map(|O| String::from_utf8_lossy(&O.stdout).trim() == "true")
					.unwrap_or(false)
			}

			#[cfg(not(target_os = "linux"))]
			{
				false
			}
		}
	};

	Ok(json!({ "dark": Dark, "highContrast": HighContrast }))
}

fn detect_dark_mode() -> bool {
	// runs once then cached via OnceLock
	#[cfg(target_os = "macos")]
	{
		std::process::Command::new("defaults")
			.args(["read", "-g", "AppleInterfaceStyle"])
			.output()
			.ok()
			.map(|O| String::from_utf8_lossy(&O.stdout).trim().to_lowercase().contains("dark"))
			.unwrap_or(false)
	}

	#[cfg(target_os = "windows")]
	{
		std::process::Command::new("reg")
			.args([
				"query",
				"HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize",
				"/v",
				"AppsUseLightTheme",
			])
			.output()
			.ok()
			.map(|O| {
				let Output = String::from_utf8_lossy(&O.stdout);
				Output.contains("0x0") || Output.contains("REG_DWORD    0")
			})
			.unwrap_or(false)
	}

	#[cfg(target_os = "linux")]
	{
		let GtkDark = std::process::Command::new("gsettings")
			.args(["get", "org.gnome.desktop.interface", "color-scheme"])
			.output()
			.ok()
			.map(|O| String::from_utf8_lossy(&O.stdout).contains("dark"))
			.unwrap_or(false);

		if GtkDark {
			return true;
		}

		let GtkTheme = std::process::Command::new("gsettings")
			.args(["get", "org.gnome.desktop.interface", "gtk-theme"])
			.output()
			.ok()
			.map(|O| String::from_utf8_lossy(&O.stdout).to_lowercase().contains("dark"))
			.unwrap_or(false);

		if GtkTheme {
			return true;
		}

		let KdeDark = std::env::var("KDE_COLOR_SCHEME")
			.ok()
			.map(|V| V.to_lowercase().contains("dark"))
			.unwrap_or(false);

		if KdeDark {
			return true;
		}

		let XfceDark = std::process::Command::new("xfconf-query")
			.args(["-c", "xsettings", "-p", "/Net/ThemeName"])
			.output()
			.ok()
			.map(|O| String::from_utf8_lossy(&O.stdout).to_lowercase().contains("dark"))
			.unwrap_or(false);

		XfceDark
	}

	#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
	{
		false
	}
}
