// File: EsmInterceptor/Hook.ts
// This file contains the Node.js ESM loader hook implementation. It runs in a separate
// thread and intercepts `import` statements. When it sees an import for "vscode",
// it communicates with the main Cocoon thread to get a dynamically generated module
// that provides the appropriate `vscode` API shim for the importing extension.

// TypeScript type definitions to provide type-safety for the Node.js loader hook APIs.
declare class MessagePort {
	on(event: "message" | "close", listener: (value: any) => void): this;
	postMessage(value: any, transferList?: ReadonlyArray<any>): void;
	close(): void;
	start?(): void;
}

interface LoaderInitializationData {
	port: MessagePort;
}

interface ResolveHookContext {
	parentURL?: string;
	conditions: string[];
	importAssertions?: Record<string, string>;
}

type NextResolveHookFunction = (
	specifier: string,
	context?: ResolveHookContext,
) => Promise<{
	url: string;
	format?: "builtin" | "commonjs" | "json" | "module" | "wasm";
	shortCircuit?: boolean;
	importAssertions?: Record<string, string>;
}>;

// --- Module State ---

let PortToMainCocoonThread: MessagePort | undefined;
let NextRequestId = 0;
const RESOLUTION_REQUEST_TIMEOUT_MILLISECONDS = 5000;
const PendingResolutionPromises = new Map<
	number,
	{
		Resolve: (resolvedUrl: string) => void;
		Reject: (reason?: any) => void;
		TimeoutId?: NodeJS.Timeout;
	}
>();

/**
 * Initializes the loader hook. This function is called by Node.js when the loader is registered.
 * It establishes a `MessagePort` communication channel with the main Cocoon thread.
 * @param DataFromRegister The data object provided by `node:module.register`, which contains the message port.
 */
export const initialize = (
	DataFromRegister: LoaderInitializationData | undefined,
): void => {
	if (!DataFromRegister?.port) {
		console.error(
			"[EsmLoaderHook] CRITICAL FAILURE: MessagePort to main Cocoon thread was not received. This ESM loader will be ineffective.",
		);
		return;
	}
	PortToMainCocoonThread = DataFromRegister.port;

	PortToMainCocoonThread.on(
		"message",
		(EventFromMainThread: {
			data: {
				id: number;
				url?: string;
				error?: { message: string };
			};
		}) => {
			const {
				id: ResponseId,
				url: ResolvedUrl,
				error: ResolutionError,
			} = EventFromMainThread.data;
			const PromiseCallbacks = PendingResolutionPromises.get(ResponseId);

			if (PromiseCallbacks) {
				if (PromiseCallbacks.TimeoutId)
					clearTimeout(PromiseCallbacks.TimeoutId);
				PendingResolutionPromises.delete(ResponseId);

				if (ResolutionError) {
					PromiseCallbacks.Reject(
						new Error(
							ResolutionError.message ||
								'Unknown error resolving "vscode" from main thread',
						),
					);
				} else if (typeof ResolvedUrl === "string") {
					PromiseCallbacks.Resolve(ResolvedUrl);
				} else {
					PromiseCallbacks.Reject(
						new Error(
							"[EsmLoaderHook] Invalid message from main thread: Missing 'url' or 'error'.",
						),
					);
				}
			} else {
				console.warn(
					`[EsmLoaderHook] Received message for unknown or already handled request ID: ${ResponseId}.`,
				);
			}
		},
	);

	PortToMainCocoonThread.on("close", () => {
		console.log(
			"[EsmLoaderHook] MessagePort to main Cocoon thread closed. This loader can no longer resolve 'vscode' imports.",
		);
		PortToMainCocoonThread = undefined;
		PendingResolutionPromises.forEach((Callbacks, RequestId) => {
			if (Callbacks.TimeoutId) clearTimeout(Callbacks.TimeoutId);
			Callbacks.Reject(
				new Error(
					`[EsmLoaderHook] Communication channel closed while request ID ${RequestId} was pending.`,
				),
			);
		});
		PendingResolutionPromises.clear();
	});

	console.log(
		"[EsmLoaderHook] Initialized successfully. Listening for responses from main Cocoon thread.",
	);
};

/**
 * The `resolve` hook. This function is called by the Node.js module loader for each `import` statement.
 * @param Specifier The module specifier being imported (e.g., "vscode", "react").
 * @param Context Metadata about the import, including the URL of the parent module.
 * @param NextResolve The next `resolve` hook in the chain, to be called if this hook doesn't handle the specifier.
 * @returns A promise that resolves to the module's URL and format.
 */
export const resolve = async (
	Specifier: string,
	Context: ResolveHookContext,
	NextResolve: NextResolveHookFunction,
): Promise<{
	url: string;
	format?: "builtin" | "commonjs" | "json" | "module" | "wasm";
	shortCircuit?: boolean;
}> => {
	const ShouldIntercept = Specifier === "vscode" || Specifier === "land"; // "land" as a potential alias

	if (!ShouldIntercept || !Context.parentURL) {
		return NextResolve(Specifier, Context);
	}

	if (!PortToMainCocoonThread) {
		console.error(
			`[EsmLoaderHook] Cannot resolve "${Specifier}": MessagePort to main thread is not available. Delegating to next resolver.`,
		);
		return NextResolve(Specifier, Context);
	}

	const ImportingModuleUrl = Context.parentURL;
	const RequestId = NextRequestId++;

	const ResolutionPromise = new Promise<string>(
		(ResolveCallback, RejectCallback) => {
			const TimeoutId = setTimeout(() => {
				if (PendingResolutionPromises.has(RequestId)) {
					PendingResolutionPromises.delete(RequestId);
					RejectCallback(
						new Error(
							`[EsmLoaderHook] Timeout (${RESOLUTION_REQUEST_TIMEOUT_MILLISECONDS}ms) waiting for main thread to resolve "${Specifier}".`,
						),
					);
				}
			}, RESOLUTION_REQUEST_TIMEOUT_MILLISECONDS);
			PendingResolutionPromises.set(RequestId, {
				Resolve: ResolveCallback,
				Reject: RejectCallback,
				TimeoutId,
			});
		},
	);

	PortToMainCocoonThread.postMessage({
		id: RequestId,
		importingModuleUrl: ImportingModuleUrl,
		requestedSpecifier: Specifier,
	});

	try {
		const DynamicApiModuleUrl = await ResolutionPromise;
		// Intercepting with `shortCircuit: true` tells Node.js that this is the definitive resolution.
		return {
			url: DynamicApiModuleUrl,
			shortCircuit: true,
			format: "module",
		};
	} catch (ResolutionError: any) {
		console.error(
			`[EsmLoaderHook] Error resolving "${Specifier}" for module ${ImportingModuleUrl}: ${ResolutionError.message}. Delegating to next resolver.`,
		);
		if (PendingResolutionPromises.has(RequestId)) {
			const PromiseEntry = PendingResolutionPromises.get(RequestId);
			if (PromiseEntry?.TimeoutId) clearTimeout(PromiseEntry.TimeoutId);
			PendingResolutionPromises.delete(RequestId);
		}
		return NextResolve(Specifier, Context);
	}
};

console.log(
	"[EsmLoaderHook] Loader hook script parsed and ready for `initialize` and `resolve` calls.",
);
