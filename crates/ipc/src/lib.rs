mod graph;
mod ipc;
mod metadata;

pub use crate::{
	graph::GraphIpc,
	ipc::{IpcError, IpcInstance, IpcMessage, IpcRx, IpcTx},
	metadata::{CompressedBasicBlock, NodeMetadata},
};
