<table>
	<tr>
		<td colspan="1">
			<h3 align="center">
				<picture>
					<source media="(prefers-color-scheme: dark)" srcset="https://editor.land/Dark/Image/GitHub/Land.svg" />
					<source media="(prefers-color-scheme: light)" srcset="https://editor.land/Image/GitHub/Land.svg" />
					<img width="28" alt="Land Logo" src="https://editor.land/Image/GitHub/Land.svg" />
				</picture>
			</h3>
		</td>
		<td colspan="3" valign="top">
			<h3 align="center">
				Mountain Binaries&#x2001;📦
			</h3>
		</td>
	</tr>
</table>

---

# **Mountain Binaries** 📦 Staged Executables for Bundling

This directory, `Binary/`, serves as a temporary staging area for executables
that are dynamically selected and prepared by the `Build.rs` orchestrator. It is
essential for managing different application "flavours" that bundle specific
sidecar runtimes.

## Purpose and Workflow

The Land Code Editor is designed to support different underlying runtimes for
its extension host, such as various versions of Node.js. It would be inefficient
and create bloated installers to bundle every possible runtime into a single
application.

To solve this, the `Binary/` directory is used as part of a "just-in-time"
bundling process:

1.  **Selection:** During the build process, the `Build.rs` script determines
    which specific sidecar version is needed (e.g., `Node.js v22`) based on the
    build arguments (`--node-version`).
2.  **Staging:** The script copies the selected executable from the main
    `SideCar/` repository (where all versions are stored) into this `Binary/`
    directory. The copied file is always given a consistent, predictable name
    (e.g., `node.exe` on Windows or `node` on Unix).
3.  **Bundling:** The `tauri.conf.json` is dynamically configured to point its
    `bundle.externalBin` to this staged file (e.g., `bin/node`). The Tauri
    bundler then includes this single executable in the final application
    installer.
4.  **Cleanup:** After the Tauri build is complete, the `Build.rs` script
    automatically cleans up this directory, removing the temporary binary.

This ensures that the final application installer is lean and contains only the
one required sidecar runtime, while the source code remains clean and doesn't
need to be polluted with temporary files.

> [!IMPORTANT]
>
> This directory is managed automatically by the build system. Its contents are
> transient and **should not be committed to version control**. It should be
> included in your project's `.gitignore` file.

---

**Parent Project**:
[`Mountain`](https://github.com/CodeEditorLand/Mountain/tree/Current/README.md)
| **Related Directory**:
[`SideCar`](https://github.com/CodeEditorLand/SideCar/tree/Current/README.md)
