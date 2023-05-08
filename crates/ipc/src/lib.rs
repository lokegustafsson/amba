mod graph;
mod ipc;
mod metadata;

pub use crate::{
	graph::GraphIpc,
	ipc::{new_wrapping, IpcError, IpcMessage, IpcRx, IpcTx},
	metadata::{CompressedBasicBlock, NodeMetadata},
};
