use std::{
	sync::{mpsc, Arc, Mutex, RwLock},
	thread,
};

use eframe::{
	egui::{self, Context},
	App, CreationContext, Frame,
};
use graphui::{EmbeddingParameters, Graph2D, GraphWidget};

use crate::{
	cmd::Cmd,
	run::{Controller, ControllerMsg},
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
	pub state_graph: RwLock<Graph2D>,
	pub block_graph: RwLock<Graph2D>,
	pub embedding_parameters: Mutex<EmbeddingParameters>,
}

impl Model {
	pub fn new() -> Self {
		Self {
			state_graph: RwLock::new(Graph2D::empty()),
			block_graph: RwLock::new(Graph2D::empty()),
			embedding_parameters: Mutex::new(EmbeddingParameters::default()),
		}
	}
}

struct Gui {
	controller_tx: mpsc::Sender<ControllerMsg>,
	model: Arc<Model>,
	graph_widget: GraphWidget,
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
		}
	}
}

impl App for Gui {
	fn update(&mut self, ctx: &Context, _: &mut Frame) {
		let graph = &self.model.block_graph.read().unwrap();
		let active = self.graph_widget.active_node_id();

		egui::TopBottomPanel::top("top-panel").show(ctx, |ui| {
			ui.horizontal(|ui| {
				ui.heading("Drawing parameters");
				let params_widget = ui.add(&mut *self.model.embedding_parameters.lock().unwrap());
				if params_widget.changed() {
					self.controller_tx
						.send(ControllerMsg::EmbeddingParamsUpdated)
						.unwrap();
				}
			})
		});
		egui::TopBottomPanel::bottom("bottom-panel").show(ctx, |ui| {
			ui.horizontal(|ui| {
				if let Some(active) = active {
					ui.heading("Selected node");
					ui.label(format!(
						"{}: {:#?}",
						active, graph.node_metadata[active]
					));
				}
			})
		});
		egui::CentralPanel::default().show(ctx, |ui| self.graph_widget.show(ui, graph));
	}

	fn on_exit(&mut self, _: Option<&eframe::glow::Context>) {
		match self.controller_tx.send(ControllerMsg::GuiShutdown) {
			Ok(()) => tracing::info!("gui telling controller to exit"),
			Err(mpsc::SendError(ControllerMsg::GuiShutdown)) => {
				tracing::warn!("controller already exited")
			}
			Err(mpsc::SendError(_)) => unreachable!(),
		}
	}
}
