use serde::{Deserialize, Serialize};

use crate::metadata::NodeMetadata;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GraphIpc {
	/// Metadata for nodes with implicit indices `0..metadata.len()`.
	pub metadata: Vec<NodeMetadata>,
	/// Directed edges between nodes, identified by their implicit indices
	pub edges: Vec<(usize, usize)>,
}
