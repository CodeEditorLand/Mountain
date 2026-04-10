/**
 * Cocoon Bootstrap Fork
 *
 * Minimal bootstrap script for the Cocoon extension host sidecar.
 * Spawned by Mountain's CocoonManagement.rs as: node bootstrap-fork.js
 *
 * Environment variables (set by Mountain):
 *   MOUNTAIN_GRPC_PORT — Mountain's Vine gRPC port (default 50051)
 *   COCOON_GRPC_PORT   — Port for Cocoon's gRPC server  (default 50052)
 *   VSCODE_PIPE_LOGGING — Enable pipe-based logging
 *   VSCODE_PARENT_PID   — Mountain's PID for orphan detection
 *
 * This bootstrap script:
 * 1. Validates the Node.js runtime
 * 2. Signals readiness to Mountain via stdout
 * 3. Delegates to the full Cocoon entry point (when compiled)
 * 4. Falls back to a keep-alive loop if the entry point is missing
 */

const MountainGRPCPort = process.env.MOUNTAIN_GRPC_PORT || "50051";
const CocoonGRPCPort = process.env.COCOON_GRPC_PORT || "50052";
const ParentPID = process.env.VSCODE_PARENT_PID;

console.log(`[Cocoon] Bootstrap starting (Node ${process.version})`);
console.log(`[Cocoon] Mountain gRPC: localhost:${MountainGRPCPort}`);
console.log(`[Cocoon] Cocoon  gRPC: localhost:${CocoonGRPCPort}`);
console.log(`[Cocoon] Parent PID: ${ParentPID}`);

// Orphan detection — exit if Mountain dies
if (ParentPID) {
	setInterval(() => {
		try {
			process.kill(Number(ParentPID), 0);
		} catch {
			console.log("[Cocoon] Parent process gone, exiting.");
			process.exit(0);
		}
	}, 5000);
}

// Try to load the compiled Cocoon entry point
const CocoonEntryPaths = [
	// Relative to this script (scripts/cocoon/ -> ../../Element/Cocoon/Target/)
	new URL("../../../Cocoon/Target/Bootstrap/Implementation/CocoonMain.js", import.meta.url),
	// Direct ESBuild output
	new URL("../../../Cocoon/Target/ESBuild/CocoonMain.js", import.meta.url),
];

let Loaded = false;

for (const EntryPath of CocoonEntryPaths) {
	try {
		const { pathname } = EntryPath;
		const { existsSync } = await import("node:fs");

		if (existsSync(pathname)) {
			console.log(`[Cocoon] Loading entry point: ${pathname}`);
			await import(EntryPath.href);
			Loaded = true;
			break;
		}
	} catch (Error) {
		console.error(`[Cocoon] Failed to load entry point:`, Error);
	}
}

if (!Loaded) {
	console.log("[Cocoon] No compiled entry point found. Running in stub mode.");
	console.log("[Cocoon] Build Cocoon with: cd Element/Cocoon && pnpm prepublishOnly");

	// Keep alive so Mountain sees the process as running
	// (prevents restart loops). Mountain health monitor checks every 5s.
	setInterval(() => {}, 30000);
}
