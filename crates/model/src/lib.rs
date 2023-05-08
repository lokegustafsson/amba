use std::{
	collections::BTreeSet,
	fmt, mem,
	ops::BitOrAssign,
	sync::{Mutex, MutexGuard, RwLock, RwLockReadGuard},
	time::Instant,
};

use graphui::{EmbeddingParameters, Graph2D, LodText};
use ipc::{CompressedBasicBlock, NodeMetadata};

use crate::control_flow::ControlFlowGraph;

mod control_flow;

/// An `Arc<Model>` is shared between the AMBA gui and embedder threads.
pub struct Model {
	// TODO: For every graph: the graph, the graph2d, the rendered node metadata
	// (controlflowgraph because we must support incremental edge adding)
	block_control_flow: RwLock<ControlFlowGraph>,
	state_control_flow: RwLock<ControlFlowGraph>,
	raw_state_graph: RwLock<Graph2D>,
	raw_block_graph: RwLock<Graph2D>,
	compressed_block_graph: RwLock<Graph2D>,
	embedding_parameters: Mutex<EmbeddingParameters>,
	/// Model supports mixed read/write, but only by a single writer.
	/// EXCLUDING `embedding_parameters` that can be written to by anyone.
	modelwide_single_writer_lock: Mutex<()>,
}

impl Model {
	pub fn new() -> Self {
		Self {
			block_control_flow: RwLock::new(ControlFlowGraph::new()),
			state_control_flow: RwLock::new(ControlFlowGraph::new()),
			raw_state_graph: RwLock::new(Graph2D::empty()),
			raw_block_graph: RwLock::new(Graph2D::empty()),
			compressed_block_graph: RwLock::new(Graph2D::empty()),
			embedding_parameters: Mutex::new(EmbeddingParameters::default()),
			modelwide_single_writer_lock: Mutex::new(()),
		}
	}

	pub fn add_new_edges(
		&self,
		state_edges: Vec<(NodeMetadata, NodeMetadata)>,
		block_edges: Vec<(NodeMetadata, NodeMetadata)>,
	) {
		let mutex: MutexGuard<'_, ()> = self.modelwide_single_writer_lock.lock().unwrap();

		{
			let mut block_control_flow = self.block_control_flow.write().unwrap();
			for (from, to) in block_edges.into_iter() {
				block_control_flow.update(from, to);
			}

			self.raw_block_graph
				.write()
				.unwrap()
				.seeded_replace_self_with({
					let (nodes, edges) = block_control_flow.get_raw_metadata_and_sequential_edges();
					(nodes.iter().map(new_lod_text).collect(), edges)
				});
			self.compressed_block_graph
				.write()
				.unwrap()
				.seeded_replace_self_with({
					let (nodes, edges) =
						block_control_flow.get_compressed_metadata_and_sequential_edges();
					(nodes.iter().map(new_lod_text).collect(), edges)
				});
		}

		{
			let mut state_control_flow = self.state_control_flow.write().unwrap();
			for (from, to) in state_edges.into_iter() {
				state_control_flow.update(from, to);
			}

			self.raw_state_graph
				.write()
				.unwrap()
				.seeded_replace_self_with({
					let (nodes, edges) = state_control_flow.get_raw_metadata_and_sequential_edges();
					(nodes.iter().map(new_lod_text).collect(), edges)
				});
		}
		mem::drop(mutex);
	}

	pub fn run_layout_iterations(&self) -> LayoutMadeProgress {
		let params: EmbeddingParameters = *self.embedding_parameters.lock().unwrap();
		let mutex: MutexGuard<'_, ()> = self.modelwide_single_writer_lock.lock().unwrap();

		let timer = Instant::now();
		let mut total_delta_pos = 0.0;
		const SUBSTEPS: usize = 100;
		for graph in [
			&self.raw_state_graph,
			&self.raw_block_graph,
			&self.compressed_block_graph,
		] {
			let mut working_copy: Graph2D = graph.read().unwrap().clone();
			total_delta_pos += working_copy.run_layout_iterations(SUBSTEPS, params);
			*graph.write().unwrap() = working_copy;
		}
		let (ret, updates_per_second, enable_repulsion_approximation) = if total_delta_pos < 0.1 {
			(LayoutMadeProgress::NoJustTiny, 0.0, true)
		} else if total_delta_pos < 100.0 {
			(
				LayoutMadeProgress::YesALittle,
				timer.elapsed().as_secs_f64().recip(),
				true,
			)
		} else {
			(
				LayoutMadeProgress::YesALot,
				timer.elapsed().as_secs_f64().recip(),
				false,
			)
		};

		mem::drop(mutex);
		{
			let mut params = self.embedding_parameters.lock().unwrap();
			params.statistic_updates_per_second = SUBSTEPS as f64 * updates_per_second;
			params.enable_repulsion_approximation = enable_repulsion_approximation;
		}
		ret
	}

	pub fn gui_get_graph(&self, which: GraphToView) -> RwLockReadGuard<'_, Graph2D> {
		match which {
			GraphToView::RawBlock => self.raw_block_graph.read().unwrap(),
			GraphToView::CompressedBlock => self.compressed_block_graph.read().unwrap(),
			GraphToView::State => self.raw_state_graph.read().unwrap(),
		}
	}

	pub fn gui_lock_params(&self) -> MutexGuard<'_, EmbeddingParameters> {
		self.embedding_parameters.lock().unwrap()
	}

	pub fn gui_get_node_description(&self, graph: GraphToView, node_index: usize) -> String {
		match graph {
			GraphToView::RawBlock => self.raw_block_graph.read(),
			GraphToView::CompressedBlock => self.compressed_block_graph.read(),
			GraphToView::State => self.raw_state_graph.read(),
		}
		.unwrap()
		.get_node_text(node_index)
		.to_owned()
	}
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GraphToView {
	RawBlock,
	CompressedBlock,
	State,
}

impl fmt::Display for GraphToView {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let s = match self {
			GraphToView::RawBlock => "Raw Basic Block Graph",
			GraphToView::CompressedBlock => "Compressed Block Graph",
			GraphToView::State => "State Graph",
		};
		f.write_str(s)
	}
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LayoutMadeProgress {
	YesALot,
	YesALittle,
	NoJustTiny,
}
impl BitOrAssign for LayoutMadeProgress {
	fn bitor_assign(&mut self, rhs: Self) {
		match (&self, &rhs) {
			(Self::NoJustTiny, _) => *self = rhs,
			(Self::YesALittle, Self::YesALot) => *self = Self::YesALot,
			(..) => {}
		}
	}
}

fn new_lod_text(metadata: &NodeMetadata) -> LodText {
	let mut ret = LodText::new();
	match metadata {
		NodeMetadata::State { symbolic_state_id } => {
			ret.coarser(symbolic_state_id.to_string());
		}
		NodeMetadata::BasicBlock {
			symbolic_state_id: state,
			basic_block_vaddr,
			basic_block_generation,
			basic_block_elf_vaddr,
			basic_block_content,
		} => {
			ret.coarser(format!("{state}\nfunctionname+addr2line"));
			ret.coarser(format!("{state}\nfunctionname"));
			ret.coarser(format!("{state}"));
		}
		NodeMetadata::CompressedBasicBlock(boxed) => {
			let CompressedBasicBlock {
				symbolic_state_ids,
				basic_block_vaddrs,
				basic_block_generations,
				basic_block_elf_vaddrs,
				basic_block_contents,
			} = &**boxed;
			assert!(!symbolic_state_ids.is_empty());
			let state_first = symbolic_state_ids.first().unwrap();
			let state_last = symbolic_state_ids.last().unwrap();
			if state_first == state_last {
				ret.coarser(format!("{state_first}\nfunctionname+addr2line"));
				ret.coarser(format!("{state_first}\nfunctionname"));
				ret.coarser(format!("{state_first}"));
			} else {
				ret.coarser(format!(
					"{state_first}-{state_last}\nfunctionname+addr2line"
				));
				ret.coarser(format!(
					"{state_first}-{state_last}\nfunctionname"
				));
				ret.coarser(format!("{state_first}-{state_last}"));
			}
		}
	}
	ret
}
