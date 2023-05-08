use std::{
	fmt, mem,
	ops::BitOrAssign,
	sync::{Mutex, MutexGuard, RwLock, RwLockReadGuard},
	time::Instant,
};

use arrayvec::ArrayVec;
use data_structures::SmallU64Set;
use graphui::{EmbeddingParameters, Graph2D};
use ipc::{CompressedBasicBlock, NodeMetadata};
use smallvec::SmallVec;

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
				.seeded_replace_self_with(
					block_control_flow.graph.len(),
					block_control_flow.graph.edge_list_sequentially_renamed(),
				);
			self.compressed_block_graph
				.write()
				.unwrap()
				.seeded_replace_self_with(
					block_control_flow.compressed_graph.len(),
					block_control_flow
						.compressed_graph
						.edge_list_sequentially_renamed(),
				);
		}

		{
			let mut state_control_flow = self.state_control_flow.write().unwrap();
			for (from, to) in state_edges.into_iter() {
				state_control_flow.update(from, to);
			}

			self.raw_state_graph
				.write()
				.unwrap()
				.seeded_replace_self_with(
					state_control_flow.graph.len(),
					state_control_flow.graph.edge_list_sequentially_renamed(),
				);
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
		let metadata = match graph {
			GraphToView::RawBlock => {
				self.block_control_flow.read().unwrap().metadata[node_index].clone()
			}
			GraphToView::CompressedBlock => {
				// Cloned because even the immutable get requires a mutable reference
				let mut cfg = self.block_control_flow.write().unwrap();
				let nodes = cfg
					.compressed_graph
					.get(node_index as u64)
					.map(|x| x.of.clone())
					.unwrap();

				merge_nodes_into_single_metadata(&nodes, &cfg)
			}
			GraphToView::State => {
				self.state_control_flow.read().unwrap().metadata[node_index].clone()
			}
		};

		format!("{}: {:#?}", node_index, metadata)
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

pub struct LodText {
	levels: ArrayVec<(String, u32, u32), 3>,
}

impl LodText {
	// TODO: Build using Addr2Line+Disassembler
	pub fn new(metadata: &NodeMetadata) -> Self {
		fn level(msg: String) -> (String, u32, u32) {
			const MAX_WIDTH: usize = 80;
			let mut width = 0;
			let mut height = 0;
			for line in msg.lines() {
				if line.len() <= MAX_WIDTH {
					width = width.max(line.len());
					height += 1;
				} else {
					width = MAX_WIDTH;
					height += (line.len() + MAX_WIDTH - 1) / MAX_WIDTH;
				}
			}
			(
				msg,
				u32::try_from(width).unwrap(),
				u32::try_from(height).unwrap(),
			)
		}
		match metadata {
			NodeMetadata::State { symbolic_state_id } => {
				let mut levels = ArrayVec::new();
				levels.push(level(symbolic_state_id.to_string()));
				Self { levels }
			}
			NodeMetadata::BasicBlock {
				symbolic_state_id,
				basic_block_vaddr,
				basic_block_generation,
				basic_block_elf_vaddr,
				basic_block_content,
			} => Self {
				levels: ArrayVec::from([
					level("TODO".to_owned()),
					level("TODO".to_owned()),
					level(symbolic_state_id.to_string()),
				]),
			},
			NodeMetadata::CompressedBasicBlock(boxed) => {
				let CompressedBasicBlock {
					symbolic_state_ids,
					basic_block_vaddrs,
					basic_block_generations,
					basic_block_elf_vaddrs,
					basic_block_contents,
				} = &**boxed;
				Self {
					levels: ArrayVec::from([
						level("TODO".to_owned()),
						level("TODO".to_owned()),
						level({
							assert_eq!(
								symbolic_state_ids.first(),
								symbolic_state_ids.last(),
							);
							assert!(!symbolic_state_ids.is_empty());
							symbolic_state_ids.first().unwrap().to_string()
						}),
					]),
				}
			}
		}
	}

	pub fn get_given_available_square(&self, width: u32, height: u32) -> &str {
		for (content, w, h) in &self.levels {
			if *w <= width && *h <= height {
				return content;
			}
		}
		""
	}
}

fn merge_nodes_into_single_metadata(nodes: &SmallU64Set, cfg: &ControlFlowGraph) -> NodeMetadata {
	let mut symbolic_state_ids = SmallVec::new();
	let mut basic_block_vaddrs = SmallVec::new();
	let mut basic_block_generations = SmallVec::new();
	let mut basic_block_elf_vaddrs = SmallVec::new();
	let mut basic_block_contents = SmallVec::new();

	for metadata in nodes.iter().map(|i| &cfg.metadata[*i as usize]) {
		if let NodeMetadata::BasicBlock {
			symbolic_state_id,
			basic_block_vaddr,
			basic_block_generation,
			basic_block_elf_vaddr,
			basic_block_content,
		} = metadata
		{
			symbolic_state_ids.push(*symbolic_state_id);
			basic_block_vaddrs.push(*basic_block_vaddr);
			basic_block_generations.push(*basic_block_generation);
			basic_block_elf_vaddrs.push(*basic_block_elf_vaddr);
			basic_block_contents.push(basic_block_content.clone());
		} else {
			panic!("Basic block graph contained non-basic-block metadata")
		};
	}

	NodeMetadata::CompressedBasicBlock(Box::new(CompressedBasicBlock {
		symbolic_state_ids,
		basic_block_vaddrs,
		basic_block_generations,
		basic_block_elf_vaddrs,
		basic_block_contents,
	}))
}
