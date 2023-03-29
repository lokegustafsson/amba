use std::{
	sync::{mpsc, Arc, Mutex},
	thread,
};

use data_structures::Graph2D;
use eframe::{egui::Context, App, CreationContext, Frame};

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
	pub state_graph: Graph2D,
	pub block_graph: Graph2D,
}

impl Model {
	pub fn new() -> Self {
		Self {
			state_graph: Graph2D::empty(),
			block_graph: Graph2D::empty(),
		}
	}
}

struct Gui {
	controller_tx: mpsc::Sender<ControllerMsg>,
	/// Asynchronously computed model, displayed by the GUI somehow
	model: Arc<Mutex<Model>>,
}

impl Gui {
	fn new(cc: &CreationContext<'_>, cmd: &'static mut Cmd, config: SessionConfig) -> Self {
		let (controller_tx, controller_rx) = mpsc::channel();
		let model = Arc::new(Mutex::new(Model::new()));

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
					})
					.run(cmd, &config)
				}
			})
			.unwrap();

		Self {
			controller_tx,
			model,
		}
	}
}

impl App for Gui {
	fn update(&mut self, ctx: &Context, frame: &mut Frame) {
		// Totally empty GUI for now
	}

	fn on_exit(&mut self, _: Option<&eframe::glow::Context>) {
		match self.controller_tx.send(ControllerMsg::Shutdown) {
			Ok(()) => tracing::info!("gui telling controller to exit"),
			Err(mpsc::SendError(ControllerMsg::Shutdown)) => {
				tracing::warn!("controller already exited")
			}
			Err(mpsc::SendError(_)) => unreachable!(),
		}
	}
}
