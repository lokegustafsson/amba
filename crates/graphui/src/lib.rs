use std::convert;

use egui::{self, Rect, Response, Sense, Ui, Widget};
use emath::Vec2;

mod embed;

pub use embed::{EmbeddingParameters, Graph2D};

impl Widget for &mut EmbeddingParameters {
	fn ui(self, ui: &mut Ui) -> Response {
		ui.vertical(|ui| {
			ui.add(
				egui::Slider::new(
					&mut self.noise,
					0.0..=EmbeddingParameters::MAX_NOISE,
				)
				.text("noise"),
			);
			ui.add(
				egui::Slider::new(
					&mut self.attraction,
					0.0..=EmbeddingParameters::MAX_ATTRACTION,
				)
				.text("attraction"),
			);
			ui.add(
				egui::Slider::new(
					&mut self.repulsion,
					0.0..=EmbeddingParameters::MAX_REPULSION,
				)
				.text("repulsion"),
			);
			ui.add(
				egui::Slider::new(
					&mut self.gravity,
					0.0..=EmbeddingParameters::MAX_GRAVITY,
				)
				.text("gravity"),
			);
		})
		.response
	}
}

pub struct GraphWidget {
	/// Linear zoom level:
	/// 1x => the graph fits within the area with some margin
	/// 10x => we are looking at a small part of the graph
	zoom: f32,
	pos: Vec2,
	active_node: Option<usize>,
}
impl Default for GraphWidget {
	fn default() -> Self {
		Self {
			zoom: 1.0,
			pos: Vec2::ZERO,
			active_node: None,
		}
	}
}

impl GraphWidget {
	pub fn show(&mut self, ui: &mut Ui, graph: &Graph2D) {
		egui::Frame::none()
			.stroke(ui.visuals().widgets.inactive.fg_stroke)
			.show(ui, |ui| {
				let height = ui.available_height();
				let width = ui.available_width();
				ui.set_clip_rect({
					let mut clip = ui.cursor();
					clip.set_height(height);
					clip.set_width(width);
					clip
				});

				let scrollarea = egui::ScrollArea::both()
					.auto_shrink([false, false])
					.scroll_offset(self.pos)
					.show_viewport(ui, |ui, viewport| {
						draw_graph(
							ui,
							self.zoom,
							&mut self.active_node,
							viewport,
							graph,
						)
					});

				let (zoom_delta, latest_pointer_pos) = ui.input(|input| {
					(
						input.zoom_delta() + input.scroll_delta.y * 2.0 / height,
						input.pointer.interact_pos(),
					)
				});

				let real_zoom_delta = if let (true, Some(hover_pos)) =
					(ui.ui_contains_pointer(), latest_pointer_pos)
				{
					let new_zoom = (self.zoom * zoom_delta).max(1.0);
					let real_zoom_delta = new_zoom / self.zoom;
					self.zoom = new_zoom;

					let hover_pos = 1.0 * (hover_pos - scrollarea.inner_rect.min);
					let new_offset = (self.pos + hover_pos) * real_zoom_delta - hover_pos;
					self.pos = new_offset;
					real_zoom_delta
				} else {
					1.0
				};
				self.pos = (self.pos - scrollarea.inner.drag_delta())
					.max(emath::Vec2::ZERO)
					.min(scrollarea.content_size * real_zoom_delta - scrollarea.inner_rect.size());
			});
	}
}

const NODE_WIDTH: f32 = 50.0;

fn draw_graph(
	ui: &mut Ui,
	zoom_level: f32,
	active_node: &mut Option<usize>,
	viewport: Rect,
	graph: &Graph2D,
) -> egui::Response {
	let offset = ui.cursor().left_top();
	let background = ui.allocate_rect(
		Rect::from_min_size(offset, ui.available_size() * zoom_level),
		egui::Sense::drag(),
	);
	let expanded_viewport = viewport.expand(NODE_WIDTH / 2.0);

	let style_widgets = &ui.visuals().widgets.clone();
	let style_selection = &ui.visuals().selection.clone();
	let mut draw_node = |pos, text, selected| {
		let rect = Rect::from_center_size(pos, egui::Vec2::new(NODE_WIDTH, NODE_WIDTH))
			.translate(offset.to_vec2());
		let resp = ui.allocate_rect(rect, Sense::click_and_drag());
		ui.put(rect, move |ui: &mut Ui| {
			egui::Frame::none()
				.fill(if selected {
					style_selection.bg_fill
				} else {
					style_widgets.hovered.bg_fill
				})
				.rounding(NODE_WIDTH / 5.0)
				.show(ui, |ui| {
					ui.label(egui::RichText::new(text).small());
				})
				.response
		});
		resp
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
	for (i, &node) in graph.node_positions.iter().enumerate() {
		if let Ok(pos_within_viewport) = translate(node) {
			let node = draw_node(pos_within_viewport, "", Some(i) == *active_node);
			if node.clicked() || node.dragged() {
				*active_node = Some(i);
			}
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
			style_widgets.hovered.fg_stroke,
		);
	}
	background
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
