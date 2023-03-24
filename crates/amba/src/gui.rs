use std::{
	sync::{mpsc, Arc, Mutex},
	thread,
};

use eframe::{egui::Context, App, CreationContext, Frame};

use crate::{cmd::Cmd, run, SessionConfig};

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

struct Model {}

impl Model {
	fn new() -> Self {
		Self {}
	}
}

struct Gui {
	processing_tx: mpsc::Sender<()>,
	/// Asynchronously computed model, displayed by the GUI somehow
	model: Arc<Mutex<Model>>,
}

impl Gui {
	fn new(cc: &CreationContext<'_>, cmd: &'static mut Cmd, config: SessionConfig) -> Self {
		let (processing_tx, processing_rx) = mpsc::channel();
		let model = Arc::new(Mutex::new(Model::new()));

		thread::Builder::new()
			.name("amba-controller".to_owned())
			.spawn({
				let gui_context = cc.egui_ctx.clone();
				let model = Arc::clone(&model);
				move || {
					(Controller {
						rx: processing_rx,
						model,
						gui_context,
					})
					.run(cmd, &config)
				}
			})
			.unwrap();

		Self {
			processing_tx,
			model,
		}
	}
}
impl App for Gui {
	fn update(&mut self, ctx: &Context, frame: &mut Frame) {
		// Totally empty GUI for now
	}
}

struct Controller {
	rx: mpsc::Receiver<()>,
	model: Arc<Mutex<Model>>,
	gui_context: Context,
}
impl Controller {
	fn run(self, cmd: &mut Cmd, config: &SessionConfig) -> Result<(), ()> {
		run::run(cmd, config)
	}
}
