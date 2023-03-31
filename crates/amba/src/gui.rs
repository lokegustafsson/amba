use std::{
	convert,
	sync::{mpsc, Arc, Mutex, RwLock},
	thread,
};

use data_structures::{EmbeddingParameters, Graph2D};
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
	/// Linear zoom level:
	/// 1x => the graph fits within the area with some margin
	/// 10x => we are looking at a small part of the graph
	graph_area_zoom: f32,
	graph_area_pos: emath::Vec2,
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
			graph_area_zoom: 1.0,
			graph_area_pos: emath::Vec2::ZERO,
		}
	}
}

impl App for Gui {
	fn update(&mut self, ctx: &Context, _: &mut Frame) {
		let (zoom_delta, scroll_delta) =
			ctx.input(|input| (input.zoom_delta(), input.scroll_delta));

		egui::CentralPanel::default().show(ctx, |ui| {
			ui.horizontal(|ui| {
				ui.heading("top stuff");
				ui.vertical(|ui| {
					let mut params = self.model.embedding_parameters.lock().unwrap();
					ui.add(egui::Slider::new(&mut params.noise, 0.0..=1000.0).text("noise"));
					ui.add(egui::Slider::new(&mut params.attraction, 0.0..=0.5).text("attraction"));
					ui.add(egui::Slider::new(&mut params.repulsion, 0.0..=100.0).text("repulsion"));
					ui.add(egui::Slider::new(&mut params.gravity, 0.0..=1.0).text("gravity"));
				})
			});
			draw_below_first(
				ui,
				|ui| {
					egui::Frame::none()
						.stroke(ui.visuals().widgets.inactive.fg_stroke)
						.show(ui, |ui| {
							ui.set_clip_rect({
								let mut clip = ui.cursor();
								clip.set_height(ui.available_height());
								clip.set_width(ui.available_width());
								clip
							});

							let scrollarea = egui::ScrollArea::both()
								.auto_shrink([false, false])
								.scroll_offset(self.graph_area_pos)
								.show_viewport(ui, |ui, viewport| {
									draw_graph(
										ui,
										self.graph_area_zoom,
										viewport,
										&self.model.block_graph.read().unwrap(),
									)
								});

							if let Some(hover_pos) = scrollarea.inner.hover_pos() {
								let new_zoom = (self.graph_area_zoom * zoom_delta).max(1.0);
								let real_zoom_delta = new_zoom / self.graph_area_zoom;
								self.graph_area_zoom = new_zoom;

								let hover_pos = hover_pos - scrollarea.inner_rect.min;
								let new_offset =
									(self.graph_area_pos + hover_pos) * real_zoom_delta - hover_pos;
								self.graph_area_pos = new_offset;

								self.graph_area_pos = (self.graph_area_pos - scroll_delta)
									.max(emath::Vec2::ZERO)
									.min(scrollarea.content_size - scrollarea.inner_rect.size());
							}
							self.graph_area_pos = (self.graph_area_pos
								- scrollarea.inner.drag_delta())
							.max(emath::Vec2::ZERO)
							.min(scrollarea.content_size - scrollarea.inner_rect.size());
						});
				},
				|ui| {
					ui.horizontal(|ui| ui.heading("bottom stuff"));
				},
			)
		});
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

const NODE_WIDTH: f32 = 50.0;

fn draw_graph(ui: &mut Ui, zoom_level: f32, viewport: Rect, graph: &Graph2D) -> egui::Response {
	let offset = ui.cursor().left_top();
	let background = ui.allocate_rect(
		Rect::from_min_size(offset, ui.available_size() * zoom_level),
		egui::Sense::drag(),
	);
	let expanded_viewport = viewport.expand(NODE_WIDTH / 2.0);

	let style = ui.visuals().widgets.hovered;
	let mut draw_node = |pos, text| {
		let rect = Rect::from_center_size(pos, egui::Vec2::new(NODE_WIDTH, NODE_WIDTH));
		ui.put(
			rect.translate(offset.to_vec2()),
			move |ui: &mut Ui| {
				egui::Frame::none()
					.fill(style.bg_fill)
					.rounding(NODE_WIDTH / 5.0)
					.show(ui, |ui| {
						ui.label(egui::RichText::new(text).small());
					})
					.response
			},
		)
	};
	let translate = |embed_space_pos: glam::DVec2| {
		let unit_square_pos = egui::Vec2::from(<[f32; 2]>::from(
			((embed_space_pos - graph.min) / (graph.max - graph.min).max_element()).as_vec2(),
		));
		let scrollarea = viewport.size();
		let scrollarea_content_width = (scrollarea.min_elem() - 2.0 * NODE_WIDTH) * 0.9;
		let scrollarea_offset = (scrollarea - emath::Vec2::splat(scrollarea_content_width)) / 2.0;
		let scrollarea_pos = scrollarea_offset + unit_square_pos * scrollarea_content_width;
		let final_pos = (scrollarea_pos * zoom_level).to_pos2();
		match expanded_viewport.contains(final_pos) {
			true => Ok(final_pos),
			false => Err(final_pos),
		}
	};
	for &node in &graph.node_positions {
		if let Ok(pos_within_viewport) = translate(node) {
			draw_node(pos_within_viewport, "");
		}
	}
	for &(a, b) in &graph.edges {
		let origin = translate(graph.node_positions[a]);
		let target = translate(graph.node_positions[b]);
		if origin.is_err() && target.is_err() {
			continue;
		}
		let origin = origin.unwrap_or_else(convert::identity);
		let target = target.unwrap_or_else(convert::identity);
		edge_arrow(
			ui.painter(),
			origin + offset.to_vec2(),
			target - origin,
			style.fg_stroke,
		);
	}
	background
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
