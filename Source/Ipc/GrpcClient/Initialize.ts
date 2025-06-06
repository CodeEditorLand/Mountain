// File: Ipc/GrpcClient/Initialize.ts
// Defines the initialization and lifecycle management for the gRPC client
// that connects from Cocoon to the Mountain backend.

import { CocoonMountainGrpcClient } from "./Client";

let SingletonClientInstance: CocoonMountainGrpcClient | null = null;
let ConnectionPromise: Promise<void> | null = null;

/**
 * Initializes and connects the singleton gRPC client to the Mountain server.
 * This function is idempotent; it will not create a new connection if one already exists or is in progress.
 * @param MountainServerAddress The address of the Mountain gRPC server to connect to.
 * @param ConnectionTimeoutMilliseconds The timeout for the connection attempt.
 */
export const Initialize = async (
	MountainServerAddress: string,
	ConnectionTimeoutMilliseconds?: number,
): Promise<void> => {
	if (SingletonClientInstance && SingletonClientInstance.isClientReady()) {
		console.log(
			"[GrpcClient Initialize] Client already initialized and connected.",
		);
		return;
	}

	if (ConnectionPromise) {
		console.log(
			"[GrpcClient Initialize] Connection initialization already in progress. Awaiting...",
		);
		return ConnectionPromise;
	}

	const PerformConnection = async () => {
		console.log(
			`[GrpcClient Initialize] Initializing and connecting to Mountain at: ${MountainServerAddress}`,
		);
		// Create a new instance if one doesn't exist. This allows retrying after a failed connection.
		if (!SingletonClientInstance) {
			try {
				SingletonClientInstance = new CocoonMountainGrpcClient(
					MountainServerAddress,
				);
			} catch (Error: any) {
				console.error(
					`[GrpcClient Initialize] Failed to instantiate CocoonMountainGrpcClient: ${Error.message}`,
				);
				throw Error;
			}
		}

		try {
			await SingletonClientInstance.connect(
				ConnectionTimeoutMilliseconds,
			);
			console.log(
				"[GrpcClient Initialize] Connection to Mountain gRPC server successful.",
			);
		} catch (Error) {
			console.error(
				"[GrpcClient Initialize] Failed to connect to Mountain gRPC server:",
				Error,
			);
			if (SingletonClientInstance) {
				SingletonClientInstance.close();
				SingletonClientInstance = null;
			}
			throw Error; // Re-throw to signal initialization failure.
		}
	};

	ConnectionPromise = PerformConnection().finally(() => {
		ConnectionPromise = null;
	});

	return ConnectionPromise;
};

/**
 * Gets the singleton gRPC client instance.
 * @returns The connected `CocoonMountainGrpcClient` instance.
 * @throws An error if the client is not initialized or not connected.
 */
export const GetClientInstance = (): CocoonMountainGrpcClient => {
	if (!SingletonClientInstance || !SingletonClientInstance.isClientReady()) {
		const ErrorMessage =
			"[GrpcClient GetClientInstance] Client is not initialized or not connected. Ensure Initialize() was called and awaited successfully.";
		console.error(ErrorMessage);
		throw new Error(ErrorMessage);
	}
	return SingletonClientInstance;
};

/**
 * Checks if the gRPC client is currently connected.
 * @returns `true` if the client is connected and ready, `false` otherwise.
 */
export const IsConnected = (): boolean => {
	return SingletonClientInstance?.isClientReady() || false;
};

/**
 * Closes the connection to the Mountain gRPC server.
 */
export const CloseConnection = async (): Promise<void> => {
	if (ConnectionPromise) {
		console.warn(
			"[GrpcClient CloseConnection] Attempting to close while a connection attempt is in progress. Awaiting connection attempt first.",
		);
		try {
			await ConnectionPromise;
		} catch (Error) {
			// The connection attempt failed, which is fine, we can proceed to ensure it's cleaned up.
		}
	}

	if (SingletonClientInstance) {
		console.log(
			"[GrpcClient CloseConnection] Closing gRPC client connection to Mountain.",
		);
		const ClientToClose = SingletonClientInstance;
		SingletonClientInstance = null;
		ClientToClose.close();
	} else {
		console.log(
			"[GrpcClient CloseConnection] Client not initialized or already closed.",
		);
	}
};
