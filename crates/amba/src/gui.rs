use std::{
	sync::{mpsc, Arc, Mutex},
	thread,
};

use data_structures::Graph2D;
use eframe::{
	egui::{self, Context, Rect, Ui},
	App, CreationContext, Frame,
};

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
						qemu_pid: None,
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
	fn update(&mut self, ctx: &Context, _: &mut Frame) {
		egui::CentralPanel::default().show(ctx, |ui| {
			ui.horizontal(|ui| ui.heading("top stuff"));
			draw_below_first(
				ui,
				|ui| {
					ui.set_clip_rect({
						let mut clip = ui.cursor();
						clip.set_height(ui.available_height());
						clip.set_width(ui.available_width());
						clip
					});
					egui::ScrollArea::both()
						.auto_shrink([false, false])
						.show_viewport(ui, |ui, viewport| {
							draw_graph(
								ui,
								viewport,
								&self.model.lock().unwrap().block_graph,
							)
						});
				},
				|ui| {
					ui.horizontal(|ui| ui.heading("bottom stuff"));
				},
			)
		});
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

const NODE_WIDTH: f32 = 50.0;

fn draw_graph(ui: &mut Ui, viewport: Rect, graph: &Graph2D) {
	let style = ui.visuals().widgets.hovered;
	let offset = ui.cursor().left_top().to_vec2();
	let mut draw_node = |pos, text| {
		let rect = Rect::from_center_size(pos, egui::Vec2::new(NODE_WIDTH, NODE_WIDTH));
		ui.put(rect.translate(offset), move |ui: &mut Ui| {
			egui::Frame::none()
				.fill(style.bg_fill)
				.rounding(NODE_WIDTH / 5.0)
				.show(ui, |ui| {
					ui.label(egui::RichText::new(text).small());
				})
				.response
		})
	};
	let normalize_pos = |pos| (pos - graph.min) / (graph.max - graph.min).min_element();
	let translate_pos = |pos: glam::DVec2| {
		egui::Pos2::from(<[f32; 2]>::from(
			NODE_WIDTH / 2.0 + (viewport.height() - NODE_WIDTH) * pos.as_vec2(),
		))
	};
	for node in &graph.nodes {
		let pos = normalize_pos(node.pos);
		draw_node(
			translate_pos(pos),
			format!("{:.2}\n{:.2}", node.pos.x, node.pos.y),
		);
	}
	for &(a, b) in &graph.edges {
		let origin = translate_pos(normalize_pos(graph.nodes[a].pos));
		let target = translate_pos(normalize_pos(graph.nodes[b].pos));
		edge_arrow(
			ui.painter(),
			origin + offset,
			target - origin,
			style.fg_stroke,
		);
	}
}

/// Draw widgets in `reversed_lower` bottom up, then draw widgets from `upper`
/// top down in the remaining middle space.
fn draw_below_first(
	ui: &mut Ui,
	upper: impl FnOnce(&mut Ui),
	reversed_lower: impl FnOnce(&mut Ui),
) {
	ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
		reversed_lower(ui);
		ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), upper);
	});
}

fn edge_arrow(
	painter: &egui::Painter,
	mut origin: egui::Pos2,
	vec: egui::Vec2,
	stroke: egui::Stroke,
) {
	let mut tip = origin + vec;
	let margin = NODE_WIDTH / 2.0 * vec / vec.abs().max_elem();
	origin += margin;
	tip -= margin;

	let rot = emath::Rot2::from_angle(std::f32::consts::TAU / 10.0);
	let tip_length = (((tip - origin).length()) / 4.0).min(NODE_WIDTH / 3.0);
	let dir = vec.normalized();
	painter.line_segment([origin, tip], stroke);
	painter.line_segment([tip, tip - tip_length * (rot * dir)], stroke);
	painter.line_segment(
		[tip, tip - tip_length * (rot.inverse() * dir)],
		stroke,
	);
}
