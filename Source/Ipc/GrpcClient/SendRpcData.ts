
// Defines the function for sending raw binary RPC data via gRPC from Cocoon to Mountain.
// This is typically used as the transport layer for VS Code's `RPCProtocol`.

import * as grpc from "@grpc/grpc-js";

import type {
	Empty as GrpcEmptyPb,
	RpcDataPayload as GrpcRpcDataPayloadPb,
	MountainServiceClient,
} from "../Generated/VineGrpcPb";
import { GetClientInstance } from "./Initialize";

const DEFAULT_RPC_DATA_TIMEOUT_MILLISECONDS = 10000;

/**
 * Sends a raw binary buffer to the Mountain gRPC service.
 * @param Buffer The `Uint8Array` buffer to send.
 * @param TimeoutMilliseconds The timeout for the call in milliseconds.
 * @throws An error if the gRPC call fails.
 */
export const SendRpcDataToMountain = async (
	Buffer: Uint8Array,
	TimeoutMilliseconds?: number,
): Promise<void> => {
	const TimeoutValue =
		TimeoutMilliseconds || DEFAULT_RPC_DATA_TIMEOUT_MILLISECONDS;
	console.debug(
		`[GrpcClient SendRpcData] >> Sending RPCData To Mountain: BufferLength=${Buffer.byteLength}, Timeout=${TimeoutValue}`,
	);

	let MountainServiceClientInstance: MountainServiceClient;
	try {
		MountainServiceClientInstance = GetClientInstance().getRawClient();
	} catch (ClientError: any) {
		console.error(
			`[GrpcClient SendRpcData] Failed to get gRPC client for RPCData: ${ClientError.message}. Data dropped.`,
		);
		throw ClientError;
	}

	const GrpcRpcDataPayload: GrpcRpcDataPayloadPb = {
		buffer: Buffer,
	} as GrpcRpcDataPayloadPb;

	const CallOptions: Partial<grpc.CallOptions> = {
		deadline: Date.now() + TimeoutValue,
	};
	const Metadata = new grpc.Metadata();

	try {
		await MountainServiceClientInstance.sendRpcDataToMountain(
			GrpcRpcDataPayload,
			Metadata,
			CallOptions,
		);
		console.debug(
			`[GrpcClient SendRpcData] RPCData (BufferLength=${Buffer.byteLength}) sent and acknowledged by Mountain.`,
		);
	} catch (Error: any) {
		console.error(
			`[GrpcClient SendRpcData] gRPC error sending RPCData: Code=${Error.code}, Details='${Error.details}'`,
		);
		const RefinedError = new Error(
			Error.details || `gRPC call SendRpcDataToMountain failed.`,
		) as any;
		RefinedError.code = Error.code;
		RefinedError.name = "MountainGrpcRpcDataError";
		throw RefinedError;
	}
};
