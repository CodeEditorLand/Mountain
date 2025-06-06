// File: Ipc/GrpcServer/Server.ts
// Defines the Cocoon gRPC server implementation that receives calls from Mountain.
// It loads the service definition from the .proto file and routes incoming calls
// to the appropriate handlers.

import * as path from "path";
import * as grpc from "@grpc/grpc-js";
import {
	loadPackageDefinition,
	type GrpcObject,
	type sendUnaryData as GrpcSendUnaryData,
	type ServerUnaryCall as GrpcServerUnaryCall,
	type ServiceDefinition,
	type UntypedServiceImplementation,
} from "@grpc/proto-loader";

import { type CancellationTokenRegistry } from "../../../cancellation-token-registry";
import type {
	CancelOperationRequest as GrpcCancelOperationRequestPb,
	Empty as GrpcEmptyPb,
	GenericNotification as GrpcGenericNotificationPb,
	GenericRequest as GrpcGenericRequestPb,
	GenericResponse as GrpcGenericResponsePb,
	RpcDataPayload as GrpcRpcDataPayloadPb,
} from "../Generated/VineGrpcPb";
import { HandleCancel } from "./HandleCancel";
import { HandleNotification } from "./HandleNotification";
import { HandleRequest } from "./HandleRequest";
import { HandleRpcData } from "./HandleRpcData";

export class CocoonVineGrpcService {
	private readonly GrpcServerInternal: grpc.Server;
	private readonly CancellationTokenRegistryInstance: CancellationTokenRegistry;
	// Callback to pipe raw RPC data to the VS Code RPCProtocol adapter.
	private readonly RpcDataCallbackForAdapter?: (data: Uint8Array) => void;

	constructor(
		Registry: CancellationTokenRegistry,
		RpcDataCallback?: (data: Uint8Array) => void,
	) {
		this.GrpcServerInternal = new grpc.Server();
		this.CancellationTokenRegistryInstance = Registry;
		this.RpcDataCallbackForAdapter = RpcDataCallback;
		this._DefineService();
	}

	private _DefineService(): void {
		const PROTO_PATH = path.join(__dirname, "../../../../../../vine.proto");

		try {
			const PackageDefinition = loadPackageDefinition(PROTO_PATH, {
				keepCase: true,
				longs: String,
				enums: String,
				defaults: true,
				oneofs: true,
			});
			const VineGrpcObject = grpc.loadPackageDefinition(PackageDefinition)
				.vine_ipc as GrpcObject;

			if (
				!VineGrpcObject ||
				!(VineGrpcObject.CocoonService as any).service
			) {
				throw new Error(
					"CocoonService definition not found in loaded proto. Check path and proto content.",
				);
			}

			const CocoonServiceDefinition = (
				VineGrpcObject.CocoonService as any
			).service as ServiceDefinition;

			this.GrpcServerInternal.addService(CocoonServiceDefinition, {
				processMountainRequest: (
					Call: GrpcServerUnaryCall<
						GrpcGenericRequestPb,
						GrpcGenericResponsePb
					>,
					Callback: GrpcSendUnaryData<GrpcGenericResponsePb>,
				) => HandleRequest(Call, Callback, this),

				sendMountainNotification: (
					Call: GrpcServerUnaryCall<
						GrpcGenericNotificationPb,
						GrpcEmptyPb
					>,
					Callback: GrpcSendUnaryData<GrpcEmptyPb>,
				) => HandleNotification(Call, Callback, this),

				sendRpcDataToCocoon: (
					Call: GrpcServerUnaryCall<
						GrpcRpcDataPayloadPb,
						GrpcEmptyPb
					>,
					Callback: GrpcSendUnaryData<GrpcEmptyPb>,
				) =>
					HandleRpcData(
						Call,
						Callback,
						this.RpcDataCallbackForAdapter,
					),

				cancelCocoonOperation: (
					Call: GrpcServerUnaryCall<
						GrpcCancelOperationRequestPb,
						GrpcEmptyPb
					>,
					Callback: GrpcSendUnaryData<GrpcEmptyPb>,
				) =>
					HandleCancel(
						Call,
						Callback,
						this.CancellationTokenRegistryInstance,
					),
			} as UntypedServiceImplementation);

			console.log(
				"[CocoonGrpcServer] CocoonService gRPC handlers defined and added to server.",
			);
		} catch (Error: any) {
			console.error(
				"[CocoonGrpcServer] CRITICAL_ERROR: Failed to load or define gRPC service from .proto:",
				Error,
			);
			throw Error;
		}
	}

	/**
	 * Binds the gRPC server to the specified address and starts listening for connections.
	 * @param Address The address to bind to (e.g., a UDS path or named pipe name).
	 */
	public async start(Address: string): Promise<void> {
		if (!this.GrpcServerInternal) {
			const ErrorMessage =
				"[CocoonGrpcServer] Server not properly initialized. Cannot start.";
			console.error(ErrorMessage);
			throw new Error(ErrorMessage);
		}

		let BindAddress: string;
		if (process.platform === "win32") {
			// gRPC-js uses a specific format for named pipes on Windows
			BindAddress = `pipe:\\\\.\\pipe\\${Address}`;
			console.log(
				`[CocoonGrpcServer] Attempting to bind to Named Pipe: ${BindAddress}`,
			);
		} else {
			// For Unix-like systems, remove any stale socket file before binding.
			try {
				if (require("fs").existsSync(Address)) {
					require("fs").unlinkSync(Address);
					console.log(
						`[CocoonGrpcServer] Removed existing UDS file: ${Address}`,
					);
				}
			} catch (Error: any) {
				console.warn(
					`[CocoonGrpcServer] Could not remove existing UDS file '${Address}': ${Error.message}. Binding might fail if stale.`,
				);
			}
			BindAddress = `unix:${Address}`;
			console.log(
				`[CocoonGrpcServer] Attempting to bind to UDS: ${BindAddress}`,
			);
		}

		return new Promise<void>((Resolve, Reject) => {
			this.GrpcServerInternal.bindAsync(
				BindAddress,
				grpc.ServerCredentials.createInsecure(),
				(Error, Port) => {
					if (Error) {
						console.error(
							`[CocoonGrpcServer] Failed to bind server on ${BindAddress} (resolved port/handle ${Port}):`,
							Error,
						);
						return Reject(Error);
					}
					try {
						this.GrpcServerInternal.start();
						console.log(
							`[CocoonGrpcServer] Cocoon's gRPC server listening on ${BindAddress} (resolved port/handle ${Port})`,
						);
						Resolve();
					} catch (StartError: any) {
						console.error(
							`[CocoonGrpcServer] Failed to start server after binding to ${BindAddress}:`,
							StartError.message,
							StartError,
						);
						Reject(StartError);
					}
				},
			);
		});
	}

	/**
	 * Attempts a graceful shutdown of the gRPC server.
	 */
	public shutdown(): Promise<void> {
		console.log(
			"[CocoonGrpcServer] Attempting to shut down Cocoon's gRPC server...",
		);
		return new Promise((Resolve, Reject) => {
			if (this.GrpcServerInternal) {
				this.GrpcServerInternal.tryShutdown((Error?: Error) => {
					if (Error) {
						console.error(
							"[CocoonGrpcServer] Error during graceful server shutdown:",
							Error,
						);
						console.log(
							"[CocoonGrpcServer] Server will be forcefully shut down.",
						);
						this.GrpcServerInternal.forceShutdown();
						Reject(Error);
					} else {
						console.log(
							"[CocoonGrpcServer] Cocoon's gRPC server gracefully shut down.",
						);
						Resolve();
					}
				});
			} else {
				console.log(
					"[CocoonGrpcServer] Server not running or instance invalid. No action taken.",
				);
				Resolve();
			}
		});
	}
}
