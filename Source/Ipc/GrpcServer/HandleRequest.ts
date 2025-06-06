
// Defines the gRPC request handler for the CocoonService. It receives requests
// from Mountain, dispatches them to the appropriate logic, and sends back a response.

import {
	status as GrpcStatus,
	type sendUnaryData,
	type ServerUnaryCall,
} from "@grpc/grpc-js";

import type {
	GenericRequest as GrpcGenericRequestPb,
	GenericResponse as GrpcGenericResponsePb,
	RpcError as GrpcRpcErrorPb,
} from "../Generated/VineGrpcPb";
import { Dispatch as DispatchMessage } from "../MessageDispatcher";
import {
	JsValueToProtoValue,
	ProtoValueToJsValue,
} from "../Util/ProtoValueConverter";
import { type CocoonVineGrpcService } from "./Server";

/**
 * Handles incoming unary gRPC requests from the Mountain backend.
 * @param Call The gRPC call object, containing request metadata and the request message.
 * @param Callback The function to call to send the response back to the client.
 * @param _ServiceInstance The instance of the gRPC service, for context if needed.
 */
export const HandleRequest = async (
	Call: GrpcServerUnaryCall<GrpcGenericRequestPb, GrpcGenericResponsePb>,
	Callback: sendUnaryData<GrpcGenericResponsePb>,
	_ServiceInstance: CocoonVineGrpcService,
): Promise<void> => {
	const GrpcRequest = Call.request;

	// Safely access properties, handling potential differences in generated PB types.
	const RequestIdentifier =
		typeof GrpcRequest.getRequestId === "function"
			? GrpcRequest.getRequestId()
			: (GrpcRequest as any).request_id;
	const MethodName =
		typeof GrpcRequest.getMethod === "function"
			? GrpcRequest.getMethod()
			: (GrpcRequest as any).method;
	const ProtoParameters =
		typeof GrpcRequest.getParams === "function"
			? GrpcRequest.getParams()
			: (GrpcRequest as any).params;

	console.log(
		`[CocoonGrpcServer HandleRequest] << ID=${RequestIdentifier}, Method='${MethodName}'`,
	);

	let ParametersAsJs: any;
	try {
		ParametersAsJs = ProtoValueToJsValue(ProtoParameters);
		if (
			console.trace &&
			ParametersAsJs !== null &&
			ParametersAsJs !== undefined
		) {
			console.trace(
				`[CocoonGrpcServer HandleRequest] Converted Params for ${MethodName}:`,
				ParametersAsJs,
			);
		}
	} catch (ConversionError: any) {
		console.error(
			`[CocoonGrpcServer HandleRequest] Error converting request params from ProtoValue for ${MethodName} (ReqID ${RequestIdentifier}):`,
			ConversionError,
		);
		const RpcErrorPayload: GrpcRpcErrorPb = {
			code: GrpcStatus.INVALID_ARGUMENT,
			message: `Failed to deserialize request parameters for method '${MethodName}': ${ConversionError.message}`,
			data: undefined,
		};
		// The type assertion is necessary because the generated types might not perfectly align with what the callback expects.
		return Callback(null, {
			request_id: RequestIdentifier,
			result: undefined,
			error: RpcErrorPayload,
		} as GrpcGenericResponsePb);
	}

	try {
		// Dispatch the request to the central logic handler.
		const ResultAsJs = await DispatchMessage(
			MethodName,
			ParametersAsJs,
			RequestIdentifier,
		);

		// Convert the result back to a Protobuf Value.
		const ProtoResult = JsValueToProtoValue(ResultAsJs);
		if (
			ResultAsJs !== undefined &&
			ProtoResult === undefined &&
			ResultAsJs !== null
		) {
			// This indicates a value that couldn't be serialized, which is an internal error.
			throw new Error(
				`Failed to serialize result for gRPC response method '${MethodName}'.`,
			);
		}

		const ResponsePayload: GrpcGenericResponsePb = {
			request_id: RequestIdentifier,
			result: ProtoResult,
			error: undefined,
		} as GrpcGenericResponsePb;

		if (console.trace && ResponsePayload.result) {
			console.trace(
				`[CocoonGrpcServer HandleRequest] Sending success response for ${MethodName} (ReqID ${RequestIdentifier}):`,
				ProtoValueToJsValue(ResponsePayload.result),
			);
		}
		Callback(null, ResponsePayload);
	} catch (Error: any) {
		console.error(
			`[CocoonGrpcServer HandleRequest] Error processing request (ID=${RequestIdentifier}, Method='${MethodName}'):`,
			Error,
		);
		const RpcErrorPayload: GrpcRpcErrorPb = {
			code:
				Error.code && typeof Error.code === "number"
					? Error.code
					: GrpcStatus.INTERNAL,
			message:
				Error.message || "Unknown error processing request in Cocoon.",
			data: JsValueToProtoValue(Error.data),
		} as GrpcRpcErrorPb;
		Callback(null, {
			request_id: RequestIdentifier,
			result: undefined,
			error: RpcErrorPayload,
		} as GrpcGenericResponsePb);
	}
};
