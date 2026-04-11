/**
 * Cocoon Bootstrap Fork
 *
 * Minimal bootstrap script for the Cocoon extension host sidecar.
 * Spawned by Mountain's CocoonManagement.rs as: node bootstrap-fork.js
 *
 * Environment variables (set by Mountain):
 *   MOUNTAIN_GRPC_PORT - Mountain's Vine gRPC port (default 50051)
 *   COCOON_GRPC_PORT   - Port for Cocoon's gRPC server  (default 50052)
 *   VSCODE_PIPE_LOGGING - Enable pipe-based logging
 *   VSCODE_PARENT_PID   - Mountain's PID for orphan detection
 *
 * Telemetry:
 *   - PostHog EU Cloud (debug only) — sends cocoon:* events
 *   - OTEL via performance marks (Node 16+ performance.mark)
 */

import { performance } from "node:perf_hooks";

const MountainGRPCPort = process.env.MOUNTAIN_GRPC_PORT || "50051";
const CocoonGRPCPort = process.env.COCOON_GRPC_PORT || "50052";
const ParentPID = process.env.VSCODE_PARENT_PID;

// ============================================================================
// Trace — performance.mark only, zero console.log in normal operation
// ============================================================================

const Trace = (Tag, Message) => {
	try { performance.mark(`land:cocoon:${Tag}:${Message}`); } catch {}
};

Trace("bootstrap", "start");

// ============================================================================
// PostHog — debug only, fire-and-forget via HTTP POST
// ============================================================================

const PostHogAPIKey = "phc_mCwHy7LgvbnEqh6a2DyMiLUJcaZvmmj7JNmmpQzvr7mA";
const PostHogHost = "https://eu.i.posthog.com";
const DistinctId = `land-dev-${process.env.USER || process.env.USERNAME || "unknown"}`;

const PostHogCapture = async (EventName, Properties = {}) => {
	if (process.env.NODE_ENV === "production") return;
	try {
		const { request } = await import("node:https");
		const Body = JSON.stringify({
			api_key: PostHogAPIKey,
			event: EventName,
			properties: {
				distinct_id: DistinctId,
				$app: "land-editor",
				$app_version: "0.0.1",
				$build_mode: "debug",
				$component: "cocoon",
				node_version: process.version,
				mountain_grpc_port: MountainGRPCPort,
				cocoon_grpc_port: CocoonGRPCPort,
				...Properties,
			},
			timestamp: new Date().toISOString(),
		});
		const URL = new globalThis.URL(`${PostHogHost}/capture/`);
		const Req = request({
			hostname: URL.hostname,
			port: 443,
			path: URL.pathname,
			method: "POST",
			headers: { "Content-Type": "application/json", "Content-Length": Buffer.byteLength(Body) },
		});
		Req.on("error", () => {}); // Swallow — fire and forget
		Req.write(Body);
		Req.end();
	} catch {
		// PostHog unavailable — no-op
	}
};

// ============================================================================
// OTEL — send performance marks to Mountain's OTLP proxy
// ============================================================================

const OTLPFlush = async () => {
	if (process.env.NODE_ENV === "production") return;
	try {
		const Entries = performance.getEntriesByType("mark").filter(E => E.name.startsWith("land:"));
		if (Entries.length === 0) return;

		const TraceId = Array.from({ length: 16 }, () =>
			Math.floor(Math.random() * 256).toString(16).padStart(2, "0"),
		).join("");

		const Payload = {
			resourceSpans: [{
				resource: {
					attributes: [
						{ key: "service.name", value: { stringValue: "land-editor-cocoon" } },
						{ key: "service.version", value: { stringValue: "0.0.1" } },
					],
				},
				scopeSpans: [{
					scope: { name: "land.cocoon.bootstrap", version: "1.0.0" },
					spans: Entries.map(E => ({
						traceId: TraceId,
						spanId: Array.from({ length: 8 }, () =>
							Math.floor(Math.random() * 256).toString(16).padStart(2, "0"),
						).join(""),
						name: E.name,
						kind: 1,
						startTimeUnixNano: String(BigInt(Math.floor(performance.timeOrigin + E.startTime)) * 1000000n),
						endTimeUnixNano: String(BigInt(Math.floor(performance.timeOrigin + E.startTime)) * 1000000n),
						status: E.name.includes("error") ? { code: 2 } : { code: 1 },
					})),
				}],
			}],
		};

		const { request } = await import("node:http");
		const Body = JSON.stringify(Payload);
		const Req = request({
			hostname: "127.0.0.1",
			port: 4318,
			path: "/v1/traces",
			method: "POST",
			headers: { "Content-Type": "application/json", "Content-Length": Buffer.byteLength(Body) },
		});
		Req.on("error", () => {});
		Req.write(Body);
		Req.end();

		performance.clearMarks();
	} catch {}
};

// Flush OTEL every 5s
setInterval(OTLPFlush, 5000);

// ============================================================================
// Orphan detection — exit if Mountain dies
// ============================================================================

if (ParentPID) {
	setInterval(() => {
		try {
			process.kill(Number(ParentPID), 0);
		} catch {
			Trace("lifecycle", "orphan-exit");
			PostHogCapture("cocoon:session:end", { reason: "orphan" });
			OTLPFlush().then(() => process.exit(0));
		}
	}, 5000);
}

// ============================================================================
// Session start
// ============================================================================

Trace("bootstrap", "session-start");
PostHogCapture("cocoon:session:start", {
	parent_pid: ParentPID,
});

// ============================================================================
// Load Cocoon entry point
// ============================================================================

const CocoonEntryPaths = [
	new URL("../../../Cocoon/Target/Bootstrap/Implementation/CocoonMain.js", import.meta.url),
	new URL("../../../Cocoon/Target/ESBuild/CocoonMain.js", import.meta.url),
];

let Loaded = false;

for (const EntryPath of CocoonEntryPaths) {
	try {
		const { pathname } = EntryPath;
		const { existsSync } = await import("node:fs");

		if (existsSync(pathname)) {
			Trace("bootstrap", "loading-entry");
			PostHogCapture("cocoon:entry:load", { path: pathname });
			await import(EntryPath.href);
			Loaded = true;
			Trace("bootstrap", "entry-loaded");
			PostHogCapture("cocoon:entry:loaded", { path: pathname });
			break;
		}
	} catch (Error) {
		Trace("bootstrap", `error:${String(Error).slice(0, 80)}`);
		PostHogCapture("cocoon:error", {
			error_tag: "entry-load",
			error_message: String(Error).slice(0, 200),
		});
	}
}

if (!Loaded) {
	Trace("bootstrap", "stub-mode");
	PostHogCapture("cocoon:stub:active", {
		reason: "no-compiled-entry-point",
		searched: CocoonEntryPaths.map(P => P.pathname),
	});

	// Keep alive — Mountain health monitor checks every 5s
	setInterval(() => {}, 30000);
}

Trace("bootstrap", "done");
