use std::{
	path::{Path, PathBuf},
	sync::{mpsc, Arc, Mutex},
	thread,
};

use eframe::{egui::Context, App, CreationContext, Frame};
use recipe::Recipe;

use crate::{cmd::Cmd, run, RunArgs, SessionDirs};

pub enum GuiConfig {
	RunRecipe(Recipe),
}

pub fn run_gui(
	config: GuiConfig,
	cmd: &'static mut Cmd,
	dependencies_dir: PathBuf,
	data_dir: PathBuf,
	args: RunArgs,
) -> Result<(), ()> {
	// Currently, the GUI requires an already created recipe.
	let recipe = match config {
		GuiConfig::RunRecipe(r) => r,
	};

	eframe::run_native(
		"amba",
		eframe::NativeOptions {
			default_theme: eframe::Theme::Light,
			..Default::default()
		},
		Box::new(move |cc| {
			Box::new(Gui::new(
				cc,
				cmd,
				dependencies_dir,
				data_dir,
				args,
			))
		}),
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
	fn new(
		cc: &CreationContext<'_>,
		cmd: &'static mut Cmd,
		dependencies_dir: PathBuf,
		data_dir: PathBuf,
		args: RunArgs,
	) -> Self {
		let (processing_tx, processing_rx) = mpsc::channel();
		let model = Arc::new(Mutex::new(Model::new()));

		let processing = thread::Builder::new()
			.name("amba-processing".to_owned())
			.spawn({
				let gui_context = cc.egui_ctx.clone();
				let model = Arc::clone(&model);
        let dependencies_dir = dependencies_dir.to_owned();
        let data_dir = data_dir.to_owned();
				move || {
					(Processing {
						rx: processing_rx,
						model,
						gui_context,
					})
					.run(cmd, &dependencies_dir, &data_dir, args)
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
		todo!()
	}
}

struct Processing {
	rx: mpsc::Receiver<()>,
	model: Arc<Mutex<Model>>,
	gui_context: Context,
}
impl Processing {
	fn run(
		self,
		cmd: &mut Cmd,
		dependencies_dir: &Path,
		data_dir: &Path,
		args: RunArgs,
	) -> Result<(), ()> {
		run::run(
			cmd,
			dependencies_dir,
			data_dir,
			SessionDirs::new(data_dir),
			args,
		)
	}
}
