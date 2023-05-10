use std::{
	collections::BTreeSet,
	fmt::{self, Debug},
	mem,
	num::NonZeroU64,
	ops::BitOrAssign,
	sync::{Mutex, MutexGuard, RwLock, RwLockReadGuard},
	time::Instant,
};

use disassembler::DisasmContext;
use graphui::{EmbeddingParameters, Graph2D, LodText};
use ipc::{CompressedBasicBlock, NodeMetadata};

use crate::control_flow::ControlFlowGraph;

/// An `Arc<Model>` is shared between the AMBA gui and embedder threads.
pub struct Model {
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
		disasm_context: &mut DisasmContext,
	) {
		let mutex: MutexGuard<'_, ()> = self.modelwide_single_writer_lock.lock().unwrap();

		let mut new_lod_text =
			|(metadata, has_self_edge)| new_lod_text_impl(&metadata, has_self_edge, disasm_context);

		{
			let mut block_control_flow = self.block_control_flow.write().unwrap();
			for (from, to) in block_edges.into_iter() {
				block_control_flow.update(from, to);
			}

			self.raw_block_graph
				.write()
				.unwrap()
				.seeded_replace_self_with({
					let (nodes, self_edge, edges) =
						block_control_flow.get_raw_metadata_and_selfedge_and_sequential_edges();
					(
						nodes
							.into_iter()
							.zip(self_edge)
							.map(&mut new_lod_text)
							.collect(),
						edges,
					)
				});
			self.compressed_block_graph
				.write()
				.unwrap()
				.seeded_replace_self_with({
					let (nodes, self_edge, edges) = block_control_flow
						.get_compressed_metadata_and_selfedge_and_sequential_edges();
					(
						nodes
							.into_iter()
							.zip(self_edge)
							.map(&mut new_lod_text)
							.collect(),
						edges,
					)
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
					let (nodes, self_edge, edges) =
						state_control_flow.get_raw_metadata_and_selfedge_and_sequential_edges();
					(
						nodes
							.into_iter()
							.zip(self_edge)
							.map(&mut new_lod_text)
							.collect(),
						edges,
					)
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
		let (ret, updates_per_second, delta_repulsion_approximation) = if total_delta_pos < 0.1 {
			(LayoutMadeProgress::NoJustTiny, 0.0, 0.0)
		} else if total_delta_pos < 100.0 {
			(
				LayoutMadeProgress::YesALittle,
				timer.elapsed().as_secs_f64().recip(),
				-0.01,
			)
		} else {
			(
				LayoutMadeProgress::YesALot,
				timer.elapsed().as_secs_f64().recip(),
				0.01,
			)
		};

		mem::drop(mutex);
		{
			let mut params = self.embedding_parameters.lock().unwrap();
			params.statistic_updates_per_second = SUBSTEPS as f64 * updates_per_second;
			params.repulsion_approximation =
				(params.repulsion_approximation + delta_repulsion_approximation).clamp(
					0.0,
					EmbeddingParameters::MAX_REPULSION_APPROXIMATION,
				);
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

	pub fn get_neighbour_states(&self, prio: usize) -> Vec<i32> {
		fn get_neighbours_inner(idx: u64, state_cfg: &ControlFlowGraph, out: &mut BTreeSet<i32>) {
			let NodeMetadata::State { s2e_state_id , .. } = state_cfg.metadata[idx as usize] else {panic!()};
			out.insert(s2e_state_id);
			let to = &state_cfg.graph.nodes[&idx].to;
			for &link in to.iter() {
				get_neighbours_inner(link, state_cfg, out);
			}
		}

		let state_cfg = self.state_control_flow.read().unwrap();
		let mut states_set = BTreeSet::new();
		get_neighbours_inner(prio as u64, &state_cfg, &mut states_set);
		states_set.into_iter().collect()
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

fn new_lod_text_impl(
	metadata: &NodeMetadata,
	has_self_edge: bool,
	disasm_context: &DisasmContext,
) -> LodText {
	let mut ret = LodText::new();
	let marker = if has_self_edge { "↺" } else { "" };
	let function_name = |elf_vaddr: Option<NonZeroU64>| {
		let elf_vaddr = elf_vaddr.map_or(0, NonZeroU64::get);
		match disasm_context.get_function_name(elf_vaddr) {
			Ok(ret) => ret,
			Err(err) => {
				tracing::warn!(
					?err,
					elf_vaddr,
					"debuginfo get_function_name failed"
				);
				format!("{:x}", elf_vaddr)
			}
		}
	};
	let block_code = |vaddr: Option<NonZeroU64>, elf_vaddr: Option<NonZeroU64>, content: &[u8]| {
		use std::fmt::Write;
		let mut elf_vaddr = elf_vaddr.map_or(0, NonZeroU64::get);
		let ins_size_and_disasm =
			disasm_context.x64_to_assembly(content, vaddr.map_or(0, NonZeroU64::get));

		let mut ret = String::new();
		for (size, disasm) in ins_size_and_disasm {
			match disasm_context.get_source_line(elf_vaddr) {
				Ok(Some(line)) => writeln!(ret, "{}", line).unwrap(),
				Ok(None) => {}
				Err(err) => {
					tracing::warn!(
						?err,
						elf_vaddr,
						"debuginfo get_function_name failed"
					);
				}
			}
			writeln!(ret, "{}", disasm).unwrap();
			elf_vaddr += size as u64;
		}
		while ret.ends_with('\n') {
			ret.pop();
		}
		ret
	};

	match metadata {
		NodeMetadata::State {
			amba_state_id,
			s2e_state_id,
		} => {
			ret.coarser(format!("{amba_state_id} ({s2e_state_id})"));
			ret.coarser(format!("{amba_state_id}"));
		}
		NodeMetadata::BasicBlock {
			symbolic_state_id: state,
			basic_block_vaddr,
			basic_block_generation: _,
			basic_block_elf_vaddr,
			basic_block_content,
			..
		} => {
			let name = function_name(*basic_block_elf_vaddr);
			let code = block_code(
				*basic_block_vaddr,
				*basic_block_elf_vaddr,
				&*basic_block_content,
			);
			ret.coarser(format!("State: {state}{marker}\n{name}\n{code}"));
			ret.coarser(format!("{state}{marker}\n{name}"));
			ret.coarser(format!("{state}{marker}"));
		}
		NodeMetadata::CompressedBasicBlock(boxed) => {
			let CompressedBasicBlock {
				symbolic_state_ids,
				basic_block_vaddrs,
				basic_block_generations: _,
				basic_block_elf_vaddrs,
				basic_block_contents,
			} = &**boxed;
			assert!(!symbolic_state_ids.is_empty());
			let first = symbolic_state_ids.first().unwrap();
			let last = symbolic_state_ids.last().unwrap();

			let name: String = basic_block_elf_vaddrs
				.iter()
				.map(|elf_vaddr| format!("{} ", function_name(*elf_vaddr)))
				.collect();
			let code: String = basic_block_vaddrs
				.iter()
				.zip(basic_block_elf_vaddrs)
				.zip(basic_block_contents)
				.map(|((vaddr, elf_vaddr), content)| {
					format!("{}\n", block_code(*vaddr, *elf_vaddr, &*content))
				})
				.collect();

			if first == last {
				ret.coarser(format!("State: {first}{marker}\n{name}\n{code}"));
				ret.coarser(format!("{first}{marker}\n{name}"));
				ret.coarser(format!("{first}{marker}"));
			} else {
				ret.coarser(format!(
					"States: {first}-{last}{marker}\n{name}\n{code}"
				));
				ret.coarser(format!("{first}-{last}{marker}\n{name}"));
				ret.coarser(format!("{first}-{last}{marker}"));
			}
		}
	}
	ret
}
