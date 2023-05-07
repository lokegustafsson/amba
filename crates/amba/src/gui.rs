use std::{
	fmt,
	sync::{mpsc, Arc, Mutex, RwLock},
	thread,
};

use data_structures::{ControlFlowGraph, SmallU64Set};
use eframe::{
	egui::{self, Context},
	App, CreationContext, Frame,
};
use graphui::{EmbeddingParameters, Graph2D, GraphWidget};
use ipc::{CompressedBasicBlock, NodeMetadata};
use smallvec::SmallVec;

use crate::{
	cmd::Cmd,
	run::control::{Controller, ControllerMsg},
	SessionConfig,
};

pub fn run_gui(cmd: &'static mut Cmd, config: SessionConfig) -> Result<(), ()> {
	eframe::run_native(
		"amba",
		eframe::NativeOptions {
			default_theme: eframe::Theme::Light,
			..Default::default()
		},
		Box::new(move |cc| Box::new(Gui::new(cc, cmd, config))),
	)
	.map_err(|error| tracing::error!(?error, "GUI"))
}

pub struct Model {
	pub block_control_flow: RwLock<ControlFlowGraph>,
	pub state_control_flow: RwLock<ControlFlowGraph>,
	pub raw_state_graph: RwLock<Graph2D>,
	pub raw_block_graph: RwLock<Graph2D>,
	pub compressed_block_graph: RwLock<Graph2D>,
	pub embedding_parameters: Mutex<EmbeddingParameters>,
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
		}
	}
}

#[derive(Debug, PartialEq)]
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
		write!(f, "{s}")
	}
}

struct Gui {
	controller_tx: mpsc::Sender<ControllerMsg>,
	model: Arc<Model>,
	graph_widget: GraphWidget,
	view: GraphToView,
}

impl Gui {
	fn new(cc: &CreationContext<'_>, cmd: &'static mut Cmd, config: SessionConfig) -> Self {
		let (controller_tx, controller_rx) = mpsc::channel();
		let model = Arc::new(Model::new());

		thread::Builder::new()
			.name("controller".to_owned())
			.spawn({
				let tx = controller_tx.clone();
				let gui_context = Some(cc.egui_ctx.clone());
				let model = Arc::clone(&model);
				move || {
					(Controller {
						tx,
						rx: controller_rx,
						model,
						gui_context,
						qemu_pid: None,
						embedder_tx: None,
					})
					.run(cmd, &config)
				}
			})
			.unwrap();

		Self {
			controller_tx,
			model,
			graph_widget: GraphWidget::default(),
			view: GraphToView::RawBlock,
		}
	}
}

impl App for Gui {
	fn update(&mut self, ctx: &Context, _: &mut Frame) {
		let graph = match self.view {
			GraphToView::RawBlock => &self.model.raw_block_graph,
			GraphToView::CompressedBlock => &self.model.compressed_block_graph,
			GraphToView::State => &self.model.raw_state_graph,
		}
		.read()
		.unwrap();

		egui::TopBottomPanel::top("top-panel").show(ctx, |ui| {
			ui.horizontal(|ui| {
				ui.heading("Drawing parameters");
				let params_widget = ui.add(&mut *self.model.embedding_parameters.lock().unwrap());
				if params_widget.changed() {
					self.controller_tx
						.send(ControllerMsg::EmbeddingParamsUpdated)
						.unwrap();
				}
				egui::ComboBox::from_label("")
					.selected_text(format!("{}", self.view))
					.show_ui(ui, |ui| {
						let first = ui.selectable_value(
							&mut self.view,
							GraphToView::RawBlock,
							"Raw Basic Block Graph",
						);
						let second = ui.selectable_value(
							&mut self.view,
							GraphToView::CompressedBlock,
							"Compressed Block Graph",
						);
						let third =
							ui.selectable_value(&mut self.view, GraphToView::State, "State Graph");

						if first.clicked() || second.clicked() || third.clicked() {
							self.graph_widget.deselect();
						}
					});
			})
		});
		if let Some(active) = self.graph_widget.active_node_id() {
			egui::TopBottomPanel::bottom("bottom-panel")
				.resizable(true)
				.max_height(ctx.screen_rect().height() * 0.6)
				.show(ctx, |ui| {
					egui::ScrollArea::vertical()
						.auto_shrink([false, true])
						.show(ui, |ui| {
							let metadata = match self.view {
								GraphToView::RawBlock => {
									self.model.block_control_flow.read().unwrap().metadata[active]
										.clone()
								}
								GraphToView::CompressedBlock => {
									// Cloned because even the immutable get requires a mutable reference
									let mut cfg = self.model.block_control_flow.write().unwrap();
									let nodes = cfg
										.compressed_graph
										.get(active as u64)
										.map(|x| x.of.clone())
										.unwrap();

									merge_nodes_into_single_metadata(&nodes, &cfg)
								}
								GraphToView::State => {
									self.model.state_control_flow.read().unwrap().metadata[active]
										.clone()
								}
							};

							ui.heading("Selected node");
							ui.label(format!("{}: {:#?}", active, metadata));
							ui.allocate_space(ui.available_size());
						});
				});
		}
		egui::CentralPanel::default().show(ctx, |ui| self.graph_widget.show(ui, &graph));
	}

	fn on_exit(&mut self, _: Option<&eframe::glow::Context>) {
		match self.controller_tx.send(ControllerMsg::GuiShutdown) {
			Ok(()) => tracing::info!("gui telling controller to exit"),
			Err(mpsc::SendError(ControllerMsg::GuiShutdown)) => {
				tracing::warn!("controller already exited");
			}
			Err(mpsc::SendError(_)) => unreachable!(),
		}
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
