use std::{
	fmt, mem,
	sync::{mpsc, Arc, Mutex, RwLock},
	thread,
};

use eframe::{
	egui::{self, Context},
	App, CreationContext, Frame,
};
use graphui::GraphWidget;
use model::{GraphToView, Model};

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
						gui_context,
						qemu_pid: None,
						embedder_tx: None,
					})
					.run(cmd, &config, model)
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
		let graph = self.model.gui_get_graph(self.view);

		egui::TopBottomPanel::top("top-panel").show(ctx, |ui| {
			ui.horizontal(|ui| {
				ui.heading("Drawing parameters");
				let params_widget = ui.add(&mut *self.model.gui_lock_params());
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
				.default_height(ctx.screen_rect().height() * 0.3)
				.max_height(ctx.screen_rect().height() * 0.6)
				.show(ctx, |ui| {
					egui::ScrollArea::vertical()
						.auto_shrink([false, true])
						.show(ui, |ui| {
							ui.heading("Selected node");
							ui.label(self.model.gui_get_node_description(self.view, active));
							ui.allocate_space(ui.available_size());
						});
				});
		}

		egui::CentralPanel::default().show(ctx, |ui| self.graph_widget.show(ui, &graph));

		if let Some(new_priority) = mem::take(&mut self.graph_widget.new_priority_node) {
			self.controller_tx
				.send(ControllerMsg::NewPriority(new_priority))
				.unwrap();
		}
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
