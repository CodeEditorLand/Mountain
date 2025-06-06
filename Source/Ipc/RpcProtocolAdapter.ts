// File: Ipc/RpcProtocolAdapter.ts
// Defines an adapter that implements the `IMessagePassingProtocol` interface
// required by VS Code's `RPCProtocol`. This adapter bridges the RPCProtocol's
// send/receive mechanism with the project's gRPC transport layer.

import { VSBuffer } from "vs/base/common/buffer";
import { Emitter, type Event } from "vs/base/common/event";
import { type IDisposable } from "vs/base/common/lifecycle";
import { type IMessagePassingProtocol } from "vs/base/parts/ipc/common/ipc";

import { SendRpcDataToMountain } from "./GrpcClient/SendRpcData";

let OnMessageEmitter: Emitter<VSBuffer> | null = null;
let OnDidDisposeEmitter: Emitter<void> | null = null;
let IsAdapterDisposed = false;

/**
 * Processes incoming raw RPC data (from gRPC) by firing the onMessage event,
 * which the `RPCProtocol` instance listens to.
 * @param Data The raw binary data received from the gRPC stream.
 */
export const ProcessIncomingRpcData = (Data: Uint8Array): void => {
	if (IsAdapterDisposed) {
		console.warn(
			"[RpcProtocolAdapter] ProcessIncomingRpcData called after adapter was disposed. Ignoring.",
		);
		return;
	}
	if (OnMessageEmitter) {
		OnMessageEmitter.fire(VSBuffer.wrap(Data));
	} else {
		// This can happen if data arrives before the RPCProtocol is fully constructed and listening.
		console.warn(
			"[RpcProtocolAdapter] Received RPC data from Mountain, but onMessageEmitter is not initialized. Data ignored.",
		);
	}
};

/**
 * Signals that the connection has been terminated, firing the onDidDispose event.
 */
export const SignalDispose = (): void => {
	if (!IsAdapterDisposed) {
		console.log(
			"[RpcProtocolAdapter] Signaling disposal of the message passing protocol.",
		);
		if (OnDidDisposeEmitter) {
			OnDidDisposeEmitter.fire();
			OnDidDisposeEmitter.dispose();
			OnDidDisposeEmitter = null;
		}
		if (OnMessageEmitter) {
			OnMessageEmitter.dispose();
			OnMessageEmitter = null;
		}
		IsAdapterDisposed = true;
	}
};

/**
 * Creates and returns an object that conforms to the `IMessagePassingProtocol`.
 * This object is then used to construct the `RPCProtocol`.
 */
export const CreateHostProtocolInterface = (): IMessagePassingProtocol => {
	if (IsAdapterDisposed) {
		console.warn(
			"[RpcProtocolAdapter] CreateHostProtocolInterface called after adapter was disposed. Reinitializing emitters.",
		);
		IsAdapterDisposed = false;
	}

	console.log(
		"[RpcProtocolAdapter] Creating host protocol interface for RPCProtocol.",
	);

	if (!OnMessageEmitter) {
		OnMessageEmitter = new Emitter<VSBuffer>();
	}
	if (!OnDidDisposeEmitter) {
		OnDidDisposeEmitter = new Emitter<void>();
	}

	const ProtocolInterface: IMessagePassingProtocol = {
		send: (BufferInstance: VSBuffer): void => {
			if (IsAdapterDisposed) {
				console.warn(
					"[RpcProtocolAdapter] send() called after adapter was disposed. Message dropped.",
				);
				return;
			}
			// Delegate sending to the gRPC client function.
			SendRpcDataToMountain(BufferInstance.buffer).catch((Error) => {
				console.error(
					"[RpcProtocolAdapter] Failed to send RPC data to Mountain via gRPC:",
					Error,
				);
				// If sending fails, it likely means the connection is broken, so we signal disposal.
				SignalDispose();
			});
		},
		onMessage: OnMessageEmitter.event,
		onDidDispose: OnDidDisposeEmitter.event,
	};

	return ProtocolInterface;
};

/**
 * Cleans up all resources used by the protocol adapter.
 */
export const Dispose = (): void => {
	SignalDispose();
};
