// File: Ipc/GrpcServer/HandleNotification.ts
// Defines the gRPC notification handler for the CocoonService. It receives
// fire-and-forget notifications from Mountain and dispatches them to the appropriate logic.

import { type sendUnaryData, type ServerUnaryCall } from "@grpc/grpc-js";
import { type Value as ProtoValue } from "google-protobuf/google/protobuf/struct_pb";

import type {
	Empty as GrpcEmptyPb,
	GenericNotification as GrpcGenericNotificationPb,
} from "../Generated/VineGrpcPb";
import { Dispatch as DispatchMessage } from "../MessageDispatcher";
import { ProtoValueToJsValue } from "../Util/ProtoValueConverter";
import { type CocoonVineGrpcService } from "./Server";

/**
 * Handles incoming unary gRPC notifications from the Mountain backend.
 * @param Call The gRPC call object, containing the notification message.
 * @param Callback The function to call to acknowledge receipt of the notification.
 * @param _ServiceInstance The instance of the gRPC service, for context if needed.
 */
export const HandleNotification = async (
	Call: GrpcServerUnaryCall<GrpcGenericNotificationPb, GrpcEmptyPb>,
	Callback: sendUnaryData<GrpcEmptyPb>,
	_ServiceInstance: CocoonVineGrpcService,
): Promise<void> => {
	const GrpcNotification = Call.request;

	// Safely access properties from the Protobuf-generated object.
	const MethodName =
		typeof GrpcNotification.getMethod === "function"
			? GrpcNotification.getMethod()
			: (GrpcNotification as any).method;
	const ProtoParameters =
		typeof GrpcNotification.getParams === "function"
			? GrpcNotification.getParams()
			: (GrpcNotification as any).params;

	console.log(
		`[CocoonGrpcServer HandleNotification] << Method='${MethodName}'`,
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
				`[CocoonGrpcServer HandleNotification] Converted Params for ${MethodName}:`,
				ParametersAsJs,
			);
		}
	} catch (ConversionError: any) {
		// Log the error but don't crash the server. Notifications are fire-and-forget.
		console.error(
			`[CocoonGrpcServer HandleNotification] Error converting notification params from ProtoValue for ${MethodName}:`,
			ConversionError,
		);
		ParametersAsJs = null; // Proceed with null parameters.
	}

	try {
		// Dispatch the notification to the central logic handler without awaiting a result.
		// We use a `void` cast to explicitly ignore any potential return value.
		void DispatchMessage(
			MethodName,
			ParametersAsJs,
			undefined, // No request ID for notifications
		);
		// Acknowledge receipt of the notification immediately.
		Callback(null, {} as GrpcEmptyPb);
	} catch (Error: any) {
		// This catch block handles synchronous errors within the DispatchMessage call itself,
		// which should be rare if it's properly async.
		console.error(
			`[CocoonGrpcServer HandleNotification] Synchronous error during dispatch of notification (Method='${MethodName}'):`,
			Error,
		);
		// Acknowledge anyway, as notifications are not meant to return errors.
		Callback(null, {} as GrpcEmptyPb);
	}
};
