use crate::Graph;

struct NodeMetadata {}

pub struct GraphIpc {
    edges: Vec<(u64, u64)>,
    metadata: Vec<NodeMetadata>,
}

impl From<&GraphIpc> for Graph {
    fn from(ipc: &GraphIpc) -> Graph {
        todo!()
    }
}
impl From<&Graph> for GraphIpc {
    fn from(graph: &Graph) -> GraphIpc {
        todo!()
    }
}
