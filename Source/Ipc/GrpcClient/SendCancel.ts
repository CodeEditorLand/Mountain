// File: Ipc/GrpcClient/SendCancel.ts
// Defines the function for sending a cancellation request via gRPC from Cocoon to Mountain.

import * as grpc from "@grpc/grpc-js";

import type {
	CancelOperationRequest as GrpcCancelOperationRequestPb,
	Empty as GrpcEmptyPb,
	MountainServiceClient,
} from "../Generated/VineGrpcPb";
import { GetClientInstance } from "./Initialize";

const DEFAULT_CANCEL_TIMEOUT_MILLISECONDS = 5000;

/**
 * Sends a request to Mountain to cancel a previously initiated operation.
 * @param RequestIdToCancel The unique identifier of the request to be cancelled.
 * @param TimeoutMilliseconds The timeout for the cancellation call in milliseconds.
 * @throws An error if the gRPC call fails.
 */
export const SendCancelToMountain = async (
	RequestIdToCancel: number,
	TimeoutMilliseconds?: number,
): Promise<void> => {
	const TimeoutValue =
		TimeoutMilliseconds || DEFAULT_CANCEL_TIMEOUT_MILLISECONDS;
	console.debug(
		`[GrpcClient SendCancel] >> Sending Cancel To Mountain: RequestIdToCancel=${RequestIdToCancel}, Timeout=${TimeoutValue}`,
	);

	let MountainServiceClientInstance: MountainServiceClient;
	try {
		MountainServiceClientInstance = GetClientInstance().getRawClient();
	} catch (ClientError: any) {
		console.error(
			`[GrpcClient SendCancel] Failed to get gRPC client for Cancel (ReqID ${RequestIdToCancel}): ${ClientError.message}. Cancel request dropped.`,
		);
		// Don't throw for a cancel request, as the original operation may have already completed.
		return;
	}

	const GrpcCancelPayload: GrpcCancelOperationRequestPb = {
		request_id_to_cancel: RequestIdToCancel,
	} as GrpcCancelOperationRequestPb;

	const CallOptions: Partial<grpc.CallOptions> = {
		deadline: Date.now() + TimeoutValue,
	};
	const Metadata = new grpc.Metadata();

	try {
		await MountainServiceClientInstance.cancelMountainOperation(
			GrpcCancelPayload,
			Metadata,
			CallOptions,
		);
		console.debug(
			`[GrpcClient SendCancel] Cancel request for RequestIdToCancel=${RequestIdToCancel} sent and acknowledged by Mountain.`,
		);
	} catch (Error: any) {
		console.error(
			`[GrpcClient SendCancel] gRPC error sending Cancel for RequestIdToCancel=${RequestIdToCancel}: Code=${Error.code}, Details='${Error.details}'`,
		);
		// We log the error but don't re-throw. The primary goal of cancellation is best-effort.
		// The original request promise will likely time out or be rejected separately if cancellation succeeds.
	}
};
