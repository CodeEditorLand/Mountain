// File: Ipc/GrpcServer/HandleRpcData.ts
// Defines the gRPC handler for receiving raw binary RPC data from Mountain,
// typically used for VS Code's `RPCProtocol`.

import { type sendUnaryData, type ServerUnaryCall } from "@grpc/grpc-js";

import type {
	Empty as GrpcEmptyPb,
	RpcDataPayload as GrpcRpcDataPayloadPb,
} from "../Generated/VineGrpcPb";

/**
 * Handles incoming gRPC messages containing raw binary RPC data.
 * @param Call The gRPC call object containing the RpcDataPayload.
 * @param Callback The function to call to acknowledge receipt.
 * @param RpcProtocolOnMessageCallback A callback function (provided during server initialization)
 *        that will process the raw buffer. This is typically the `RPCProtocol._receiveMessage` method.
 */
export const HandleRpcData = async (
	Call: GrpcServerUnaryCall<GrpcRpcDataPayloadPb, GrpcEmptyPb>,
	Callback: sendUnaryData<GrpcEmptyPb>,
	RpcProtocolOnMessageCallback?: (data: Uint8Array) => void,
): Promise<void> => {
	const GrpcRpcData = Call.request;

	// Safely access the buffer property from the Protobuf-generated object.
	const Buffer =
		typeof GrpcRpcData.getBuffer === "function"
			? GrpcRpcData.getBuffer()
			: (GrpcRpcData as any).buffer;
	const BufferLength = Buffer ? Buffer.length : 0;

	console.log(
		`[CocoonGrpcServer HandleRpcData] << Received RpcDataPayload from Mountain, BufferLength=${BufferLength}`,
	);

	if (Buffer && Buffer.length > 0) {
		if (
			RpcProtocolOnMessageCallback &&
			typeof RpcProtocolOnMessageCallback === "function"
		) {
			try {
				RpcProtocolOnMessageCallback(Buffer as Uint8Array);
				console.debug(
					`[CocoonGrpcServer HandleRpcData] Buffer (Length: ${BufferLength}) passed to rpcProtocolOnMessageCallback.`,
				);
			} catch (Error: any) {
				console.error(
					`[CocoonGrpcServer HandleRpcData] Error invoking rpcProtocolOnMessageCallback for RpcData:`,
					Error,
				);
			}
		} else {
			console.warn(
				`[CocoonGrpcServer HandleRpcData] Received RpcDataPayload, but no rpcProtocolOnMessageCallback is configured. Buffer (Length: ${BufferLength}) ignored.`,
			);
		}
	} else {
		console.warn(
			`[CocoonGrpcServer HandleRpcData] Received RpcDataPayload with empty or missing buffer. No action taken.`,
		);
	}

	// Acknowledge receipt of the data immediately.
	Callback(null, {} as GrpcEmptyPb);
};
