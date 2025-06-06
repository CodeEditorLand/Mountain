
// Defines the gRPC handler for receiving cancellation requests from Mountain.

import { type sendUnaryData, type ServerUnaryCall } from "@grpc/grpc-js";

import { type CancellationTokenRegistry } from "../../../cancellation-token-registry";
import type {
	CancelOperationRequest as GrpcCancelOperationRequestPb,
	Empty as GrpcEmptyPb,
} from "../Generated/VineGrpcPb";

/**
 * Handles incoming gRPC messages requesting the cancellation of an ongoing operation.
 * @param Call The gRPC call object containing the ID of the request to cancel.
 * @param Callback The function to call to acknowledge receipt.
 * @param Registry The CancellationTokenRegistry instance used to manage cancellation tokens.
 */
export const HandleCancel = async (
	Call: GrpcServerUnaryCall<GrpcCancelOperationRequestPb, GrpcEmptyPb>,
	Callback: sendUnaryData<GrpcEmptyPb>,
	Registry: CancellationTokenRegistry | null,
): Promise<void> => {
	const GrpcCancelRequest = Call.request;

	// Safely access the request ID property.
	const RequestIdToCancel =
		typeof GrpcCancelRequest.getRequestIdToCancel === "function"
			? GrpcCancelRequest.getRequestIdToCancel()
			: (GrpcCancelRequest as any).request_id_to_cancel;

	console.log(
		`[CocoonGrpcServer HandleCancel] << Received CancelCocoonOperation for RequestIdToCancel=${RequestIdToCancel}`,
	);

	if (!Registry) {
		console.error(
			`[CocoonGrpcServer HandleCancel] CancellationTokenRegistry is not available. Cannot process cancellation for RequestIdToCancel=${RequestIdToCancel}.`,
		);
		// Acknowledge the request even if we can't act on it.
		return Callback(null, {} as GrpcEmptyPb);
	}

	try {
		// Trigger the cancellation for the given token ID.
		Registry.Cancel(RequestIdToCancel);
		console.log(
			`[CocoonGrpcServer HandleCancel] Cancellation signal processed for RequestIdToCancel=${RequestIdToCancel} via CancellationTokenRegistry.`,
		);
	} catch (Error: any) {
		console.error(
			`[CocoonGrpcServer HandleCancel] Error during CancellationTokenRegistry.Cancel for RequestIdToCancel=${RequestIdToCancel}:`,
			Error,
		);
	}

	// Acknowledge receipt of the cancellation request.
	Callback(null, {} as GrpcEmptyPb);
};
