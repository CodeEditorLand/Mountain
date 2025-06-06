
// Defines the initialization and lifecycle management for the Cocoon gRPC server.

import { CancellationTokenRegistry } from "../../../cancellation-token-registry";
import { CocoonVineGrpcService } from "./Server";

let ServerInstance: CocoonVineGrpcService | null = null;
let ServerStartupPromise: Promise<void> | null = null;
let IsServerRunningFlag: boolean = false;

/**
 * Initializes and starts the Cocoon gRPC server.
 * This function is idempotent; it will not start a new server if one is already running or starting.
 * @param CocoonServerAddress The address for the server to listen on (e.g., a UDS path or named pipe).
 * @param Registry The CancellationTokenRegistry instance for handling cancellation requests.
 * @param RpcDataCallback A callback to process raw binary RPC data from Mountain.
 */
export const Initialize = async (
	CocoonServerAddress: string,
	Registry: CancellationTokenRegistry,
	RpcDataCallback: (data: Uint8Array) => void,
): Promise<void> => {
	if (IsServerRunningFlag && ServerInstance) {
		console.log(
			"[CocoonGrpcServer Initialize] Server already initialized and running.",
		);
		return;
	}

	if (ServerStartupPromise) {
		console.log(
			"[CocoonGrpcServer Initialize] Server startup already in progress. Awaiting existing promise.",
		);
		return ServerStartupPromise;
	}

	const PerformInitialization = async () => {
		console.log(
			`[CocoonGrpcServer Initialize] Initializing gRPC server to listen at: ${CocoonServerAddress}`,
		);
		const ServiceImplementation = new CocoonVineGrpcService(
			Registry,
			RpcDataCallback,
		);

		try {
			await ServiceImplementation.start(CocoonServerAddress);
			ServerInstance = ServiceImplementation;
			IsServerRunningFlag = true;
			console.log(
				`[CocoonGrpcServer Initialize] Cocoon's gRPC server successfully started on ${CocoonServerAddress}.`,
			);
		} catch (Error: any) {
			ServerInstance = null;
			IsServerRunningFlag = false;
			console.error(
				`[CocoonGrpcServer Initialize] CRITICAL_ERROR: Failed to start Cocoon's gRPC server on ${CocoonServerAddress}:`,
				Error,
			);
			throw Error;
		}
	};

	ServerStartupPromise = PerformInitialization().finally(() => {
		ServerStartupPromise = null;
	});

	return ServerStartupPromise;
};

/**
 * Gets the current running instance of the gRPC service.
 * @returns The `CocoonVineGrpcService` instance or `null` if not running.
 */
export const Get = (): CocoonVineGrpcService | null => {
	if (!IsServerRunningFlag || !ServerInstance) {
		console.warn(
			"[CocoonGrpcServer Get] Server is not currently running or initialized.",
		);
	}
	return ServerInstance;
};

/**
 * Checks if the gRPC server is currently running.
 * @returns `true` if the server is running, `false` otherwise.
 */
export const IsRunning = (): boolean => {
	return IsServerRunningFlag && ServerInstance !== null;
};

/**
 * Shuts down the gRPC server gracefully.
 */
export const Shutdown = async (): Promise<void> => {
	if (ServerStartupPromise) {
		console.warn(
			"[CocoonGrpcServer Shutdown] Attempting to shut down while a startup is in progress. Awaiting startup first.",
		);
		try {
			await ServerStartupPromise;
		} catch (Error) {
			// Startup failed, proceed with shutdown cleanup.
		}
	}

	if (ServerInstance && IsServerRunningFlag) {
		console.log(
			"[CocoonGrpcServer Shutdown] Attempting to shut down Cocoon's gRPC server...",
		);
		const ServerToShutDown = ServerInstance;
		ServerInstance = null;
		IsServerRunningFlag = false;
		try {
			await ServerToShutDown.shutdown();
			console.log(
				"[CocoonGrpcServer Shutdown] Cocoon's gRPC server shut down successfully.",
			);
		} catch (Error: any) {
			console.error(
				`[CocoonGrpcServer Shutdown] Error during gRPC server shutdown: ${Error.message}`,
				Error,
			);
		}
	} else {
		console.log(
			"[CocoonGrpcServer Shutdown] Server not running or already shut down. No action taken.",
		);
	}
};
