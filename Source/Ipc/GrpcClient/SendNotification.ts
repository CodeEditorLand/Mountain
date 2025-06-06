// File: Ipc/GrpcClient/SendNotification.ts
// Defines the function for sending a fire-and-forget gRPC notification from Cocoon to Mountain.

import * as grpc from "@grpc/grpc-js";

import type {
	Empty as GrpcEmptyPb,
	GenericNotification as GrpcGenericNotificationPb,
	MountainServiceClient,
} from "../Generated/VineGrpcPb";
import { JsValueToProtoValue } from "../Util/ProtoValueConverter";
import { GetClientInstance } from "./Initialize";

const DEFAULT_NOTIFICATION_TIMEOUT_MILLISECONDS = 5000;

/**
 * Sends a fire-and-forget notification to the Mountain gRPC service.
 * @param Method The name of the RPC notification method to invoke on the Mountain side.
 * @param Parameters The parameters for the RPC notification.
 * @param TimeoutMilliseconds The timeout for the call in milliseconds.
 */
export const SendNotificationToMountain = async (
	Method: string,
	Parameters: any,
	TimeoutMilliseconds?: number,
): Promise<void> => {
	const TimeoutValue =
		TimeoutMilliseconds || DEFAULT_NOTIFICATION_TIMEOUT_MILLISECONDS;
	console.debug(
		`[GrpcClient SendNotification] >> To Mountain: Method='${Method}', Timeout=${TimeoutValue}`,
	);
	if (console.trace && Parameters !== undefined) {
		console.trace(
			`[GrpcClient SendNotification] Params for ${Method}:`,
			Parameters,
		);
	}

	let MountainServiceClientInstance: MountainServiceClient;
	try {
		MountainServiceClientInstance = GetClientInstance().getRawClient();
	} catch (ClientError: any) {
		console.error(
			`[GrpcClient SendNotification] Failed to get gRPC client for method '${Method}': ${ClientError.message}. Notification dropped.`,
		);
		// Don't throw for notifications, just log and return.
		return;
	}

	const ProtoParameters = JsValueToProtoValue(Parameters);
	if (
		Parameters !== undefined &&
		ProtoParameters === undefined &&
		Parameters !== null
	) {
		const SerializationError = new Error(
			`Failed to serialize parameters for gRPC notification method '${Method}'. Check console for details.`,
		);
		console.error(
			`[GrpcClient SendNotification] ${SerializationError.message}`,
			Parameters,
		);
		// Don't throw, just log.
		return;
	}

	const GrpcNotificationPayload: GrpcGenericNotificationPb = {
		method: Method,
		params: ProtoParameters,
	} as GrpcGenericNotificationPb; // Type assertion may be needed for generated types

	const CallOptions: Partial<grpc.CallOptions> = {
		deadline: Date.now() + TimeoutValue,
	};
	const Metadata = new grpc.Metadata();

	try {
		await MountainServiceClientInstance.sendCocoonNotification(
			GrpcNotificationPayload,
			Metadata,
			CallOptions,
		);
		console.debug(
			`[GrpcClient SendNotification] Notification Method='${Method}' sent and acknowledged by Mountain's gRPC layer.`,
		);
	} catch (Error: any) {
		console.error(
			`[GrpcClient SendNotification] gRPC error sending notification (Method='${Method}'): Code=${Error.code}, Details='${Error.details}'`,
		);
		// Notifications are fire-and-forget, so we log the error but don't re-throw to the caller.
	}
};
