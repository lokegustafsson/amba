use std::{
	collections::BTreeSet,
	fmt::{self, Debug},
	mem,
	num::NonZeroU64,
	sync::{
		atomic::{AtomicU8, Ordering as MemoryOrdering},
		Mutex, MutexGuard, RwLock, RwLockReadGuard,
	},
	time::Instant,
};

use disassembler::DisasmContext;
use graphui::{EmbedderHasConverged, EmbeddingParameters, Graph2D, LodText, NodeDrawingData};
use ipc::{CompressedBasicBlock, NodeMetadata};

use crate::control_flow::ControlFlowGraph;

/// An `Arc<Model>` is shared between the AMBA gui and embedder threads.
pub struct Model {
	block_control_flow: RwLock<ControlFlowGraph>,
	merged_control_flow: RwLock<ControlFlowGraph>,
	state_control_flow: RwLock<ControlFlowGraph>,
	raw_state_graph: RwLock<Graph2D>,
	raw_block_graph: RwLock<Graph2D>,
	compressed_block_graph: RwLock<Graph2D>,
	merged_block_graph: RwLock<Graph2D>,
	merged_compressed_block_graph: RwLock<Graph2D>,
	embedding_parameters: Mutex<EmbeddingParameters>,
	graph_to_view: AtomicU8,
	/// Model supports mixed read/write, but only by a single writer.
	/// EXCLUDING `embedding_parameters` that can be written to by anyone.
	modelwide_single_writer_lock: Mutex<()>,
}

impl Model {
	pub fn new() -> Self {
		Self {
			block_control_flow: RwLock::new(ControlFlowGraph::new()),
			merged_control_flow: RwLock::new(ControlFlowGraph::new()),
			state_control_flow: RwLock::new(ControlFlowGraph::new()),
			raw_state_graph: RwLock::new(Graph2D::empty()),
			raw_block_graph: RwLock::new(Graph2D::empty()),
			compressed_block_graph: RwLock::new(Graph2D::empty()),
			merged_block_graph: RwLock::new(Graph2D::empty()),
			merged_compressed_block_graph: RwLock::new(Graph2D::empty()),
			embedding_parameters: Mutex::new(EmbeddingParameters::default()),
			graph_to_view: AtomicU8::new(GraphToView::RawBlock as u8),
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

		{
			let mut block_control_flow = self.block_control_flow.write().unwrap();
			let mut merged_control_flow = self.merged_control_flow.write().unwrap();
			for (mut from, mut to) in block_edges.into_iter() {
				block_control_flow.update(from.clone(), to.clone());
				from.reset_state();
				to.reset_state();
				merged_control_flow.update(from, to);
			}

			let (raw_nodes, raw_edges) = {
				let (nodes, self_edge, edges) =
					block_control_flow.get_raw_metadata_and_selfedge_and_sequential_edges();
				let scc_groups = block_control_flow.graph.inverse_tarjan();
				(
					nodes
						.into_iter()
						.zip(self_edge)
						.enumerate()
						.map(|(idx, (metadata, has_self_edge))| {
							let NodeMetadata::BasicBlock {
								symbolic_state_id,
								..
							} = metadata else {panic!()};
							NodeDrawingData {
								state: symbolic_state_id as _,
								scc_group: scc_groups[&idx],
								function: 0,
								lod_text: new_lod_text_impl(
									&metadata,
									has_self_edge,
									disasm_context,
								),
							}
						})
						.collect(),
					edges,
				)
			};
			self.raw_block_graph
				.write()
				.unwrap()
				.seeded_replace_self_with(raw_nodes, raw_edges);

			let (merged_nodes, merged_edges) = {
				let (nodes, self_edge, edges) =
					merged_control_flow.get_raw_metadata_and_selfedge_and_sequential_edges();
				let scc_groups = merged_control_flow.graph.inverse_tarjan();
				(
					nodes
						.into_iter()
						.zip(self_edge)
						.enumerate()
						.map(|(idx, (metadata, has_self_edge))| {
							let NodeMetadata::BasicBlock {
								symbolic_state_id,
								..
							} = metadata else {panic!()};
							NodeDrawingData {
								state: symbolic_state_id as _,
								scc_group: scc_groups[&idx],
								function: 0,
								lod_text: new_lod_text_impl(
									&metadata,
									has_self_edge,
									disasm_context,
								),
							}
						})
						.collect(),
					edges,
				)
			};
			self.merged_block_graph
				.write()
				.unwrap()
				.seeded_replace_self_with(merged_nodes, merged_edges);

			let (compressed_nodes, compressed_edges) = {
				let (nodes, self_edge, edges) =
					block_control_flow.get_compressed_metadata_and_selfedge_and_sequential_edges();
				let scc_groups = block_control_flow.compressed_graph.inverse_tarjan();
				(
					nodes
						.into_iter()
						.zip(self_edge)
						.enumerate()
						.map(|(idx, (metadata, has_self_edge))| {
							let NodeMetadata::CompressedBasicBlock(ref block) = metadata else {panic!()};
							let state =
								block.symbolic_state_ids.first().copied().unwrap_or(0) as usize;
							NodeDrawingData {
								state,
								scc_group: scc_groups[&idx],
								function: 0,
								lod_text: new_lod_text_impl(
									&metadata,
									has_self_edge,
									disasm_context,
								),
							}
						})
						.collect(),
					edges,
				)
			};
			self.compressed_block_graph
				.write()
				.unwrap()
				.seeded_replace_self_with(compressed_nodes, compressed_edges);

			let (merged_compressed_nodes, merged_compressed_edges) = {
				let (nodes, self_edge, edges) =
					merged_control_flow.get_compressed_metadata_and_selfedge_and_sequential_edges();
				(
					nodes
						.into_iter()
						.zip(self_edge)
						.enumerate()
						.map(|(idx, (metadata, has_self_edge))| {
							let NodeMetadata::CompressedBasicBlock(ref block) = metadata else {panic!()};
							let state =
								block.symbolic_state_ids.first().copied().unwrap_or(0) as usize;
							let scc_groups = merged_control_flow.compressed_graph.inverse_tarjan();
							NodeDrawingData {
								state,
								scc_group: scc_groups[&idx],
								function: 0,
								lod_text: new_lod_text_impl(
									&metadata,
									has_self_edge,
									disasm_context,
								),
							}
						})
						.collect(),
					edges,
				)
			};
			self.merged_compressed_block_graph
				.write()
				.unwrap()
				.seeded_replace_self_with(merged_compressed_nodes, merged_compressed_edges);
		}

		{
			let mut state_control_flow = self.state_control_flow.write().unwrap();
			for (from, to) in state_edges.into_iter() {
				state_control_flow.update(from, to);
			}

			let (state_nodes, state_edges) = {
				let (nodes, self_edge, edges) =
					state_control_flow.get_raw_metadata_and_selfedge_and_sequential_edges();
				(
					nodes
						.into_iter()
						.zip(self_edge)
						.map(|(metadata, has_self_edge)| NodeDrawingData {
							state: 0,
							scc_group: 0,
							function: 0,
							lod_text: new_lod_text_impl(&metadata, has_self_edge, disasm_context),
						})
						.collect(),
					edges,
				)
			};
			self.raw_state_graph
				.write()
				.unwrap()
				.seeded_replace_self_with(state_nodes, state_edges);
		}
		mem::drop(mutex);
	}

	pub fn run_layout_iterations(&self) -> EmbedderHasConverged {
		let params: EmbeddingParameters = *self.embedding_parameters.lock().unwrap();
		let graph_to_view = GraphToView::from_raw(self.graph_to_view.load(MemoryOrdering::SeqCst));
		let mutex: MutexGuard<'_, ()> = self.modelwide_single_writer_lock.lock().unwrap();

		let timer = Instant::now();
		let mut all_converged = EmbedderHasConverged::Yes;
		const SUBSTEPS: usize = 100;
		{
			let graph: &RwLock<Graph2D> = match graph_to_view {
				GraphToView::RawBlock => &self.raw_block_graph,
				GraphToView::CompressedBlock => &self.compressed_block_graph,
				GraphToView::State => &self.raw_state_graph,
				GraphToView::MergedBlock => &self.merged_block_graph,
				GraphToView::CompressedMergedBlock => &self.merged_compressed_block_graph,
			};
			let mut working_copy: Graph2D = graph.read().unwrap().clone();
			working_copy.set_params(params);
			all_converged = all_converged.and_also(working_copy.run_layout_iterations(SUBSTEPS));
			*graph.write().unwrap() = working_copy;
		}
		let updates_per_second = match all_converged {
			EmbedderHasConverged::Yes => 0.0,
			EmbedderHasConverged::No => timer.elapsed().as_secs_f64().recip(),
		};

		mem::drop(mutex);
		{
			let mut params = self.embedding_parameters.lock().unwrap();
			params.statistic_updates_per_second = SUBSTEPS as f64 * updates_per_second;
		}
		all_converged
	}

	pub fn gui_set_graph_to_view(&self, which: GraphToView) {
		self.graph_to_view
			.store(which as u8, MemoryOrdering::SeqCst);
	}

	pub fn gui_get_graph(&self, which: GraphToView) -> RwLockReadGuard<'_, Graph2D> {
		self.gui_set_graph_to_view(which);
		match which {
			GraphToView::RawBlock => self.raw_block_graph.read().unwrap(),
			GraphToView::CompressedBlock => self.compressed_block_graph.read().unwrap(),
			GraphToView::State => self.raw_state_graph.read().unwrap(),
			GraphToView::MergedBlock => self.merged_block_graph.read().unwrap(),
			GraphToView::CompressedMergedBlock => {
				self.merged_compressed_block_graph.read().unwrap()
			}
		}
	}

	pub fn gui_lock_params(&self) -> MutexGuard<'_, EmbeddingParameters> {
		self.embedding_parameters.lock().unwrap()
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
#[repr(u8)]
pub enum GraphToView {
	RawBlock = 0,
	CompressedBlock = 1,
	State = 2,
	MergedBlock = 3,
	CompressedMergedBlock = 4,
}

impl fmt::Display for GraphToView {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let s = match self {
			GraphToView::RawBlock => "Raw Basic Block Graph",
			GraphToView::CompressedBlock => "Compressed Block Graph",
			GraphToView::State => "State Graph",
			GraphToView::MergedBlock => "Merged Basic Block Graph",
			GraphToView::CompressedMergedBlock => "Compressed Merged Basic Block Graph",
		};
		f.write_str(s)
	}
}

impl GraphToView {
	fn from_raw(num: u8) -> Self {
		match num {
			0 => Self::RawBlock,
			1 => Self::CompressedBlock,
			2 => Self::State,
			3 => Self::MergedBlock,
			4 => Self::CompressedMergedBlock,
			d => panic!("invalid discriminant {d}"),
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
	let block_source_and_disasm =
		|vaddr: Option<NonZeroU64>, elf_vaddr: Option<NonZeroU64>, content: &[u8]| {
			use std::fmt::Write;
			let mut elf_vaddr = elf_vaddr.map_or(0, NonZeroU64::get);
			let ins_size_and_disasm =
				disasm_context.x64_to_assembly(content, vaddr.map_or(0, NonZeroU64::get));

			let mut source = String::new();
			let mut disassembly = String::new();
			for (size, disasm) in ins_size_and_disasm {
				match disasm_context.get_source_line(elf_vaddr) {
					Ok(Some(line)) => {
						// Force exact indentation
						let line = line.trim();
						const INDENT: &str = ";   ";
						if !source[..source.len().saturating_sub(1)].ends_with(line) {
							writeln!(source, "{INDENT}{}", line).unwrap();
							writeln!(disassembly, "{INDENT}{}", line).unwrap();
						}
					}
					Ok(None) | Err(_) => {}
				}
				writeln!(disassembly, "{}", disasm).unwrap();
				elf_vaddr += size as u64;
			}
			while source.ends_with('\n') {
				source.pop();
			}
			while disassembly.ends_with('\n') {
				disassembly.pop();
			}
			(source, disassembly)
		};

	match metadata {
		NodeMetadata::State {
			amba_state_id,
			s2e_state_id,
			concrete_inputs,
		} => {
			use std::fmt::Write;

			let mut full = format!("{amba_state_id} ({s2e_state_id})\n");
			for (var_name, var_value) in concrete_inputs {
				write!(
					full,
					"\n{var_name}:\n=\t{:?}\n=\t{}",
					&var_value,
					String::from_utf8_lossy(&var_value)
				)
				.unwrap();
			}
			ret.coarser(full);
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
			let (source, disasm) = block_source_and_disasm(
				*basic_block_vaddr,
				*basic_block_elf_vaddr,
				basic_block_content,
			);
			ret.coarser(format!(
				"State: {state}{marker}\nWithin function: {name}\n{disasm}"
			));
			ret.coarser(format!(
				"State: {state}{marker}\nWithin function: {name}\n{source}"
			));
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
			use std::fmt::Write;

			assert!(!symbolic_state_ids.is_empty());
			let first = symbolic_state_ids.first().unwrap();
			let last = symbolic_state_ids.last().unwrap();

			let mut names = String::new();
			let mut sources = String::new();
			let mut disasms = String::new();
			for ((&vaddr, &elf_vaddr), content) in basic_block_vaddrs
				.iter()
				.zip(basic_block_elf_vaddrs)
				.zip(basic_block_contents)
			{
				let name = function_name(elf_vaddr);
				let (source, disasm) = block_source_and_disasm(vaddr, elf_vaddr, &*content);
				if !names.ends_with(&name) {
					write!(names, " {name}").unwrap();
				}
				write!(sources, "\n\n{name}:\n{source}").unwrap();
				write!(disasms, "\n\n{name}:\n{disasm}").unwrap();
			}

			if first == last {
				ret.coarser(format!(
					"State: {first}{marker}\nWithin functions: {names}{disasms}"
				));
				ret.coarser(format!(
					"State: {first}{marker}\nWithin functions: {names}{sources}"
				));
				ret.coarser(format!("{first}{marker}\n{names}"));
				ret.coarser(format!("{first}{marker}"));
			} else {
				ret.coarser(format!(
					"States: {first}-{last}{marker}\nWithin functions: {names}{disasms}"
				));
				ret.coarser(format!(
					"States: {first}-{last}{marker}\nWithin functions: {names}{sources}"
				));
				ret.coarser(format!("{first}-{last}{marker}\n{names}"));
				ret.coarser(format!("{first}-{last}{marker}"));
			}
		}
	}
	ret
}
