use egui::{self, Rect, Response, Sense, Ui, Widget};
use emath::Vec2;

mod embed;

pub use embed::{EmbeddingParameters, Graph2D};

impl Widget for &mut EmbeddingParameters {
	fn ui(self, ui: &mut Ui) -> Response {
		const STEPS: f64 = 10.0;
		ui.vertical(|ui| {
			let resp = ui
				.add(
					egui::Slider::new(
						&mut self.noise,
						0.0..=EmbeddingParameters::MAX_NOISE,
					)
					.step_by(EmbeddingParameters::MAX_NOISE / STEPS)
					.text("noise"),
				)
				.union(
					ui.add(
						egui::Slider::new(
							&mut self.attraction,
							(EmbeddingParameters::MAX_ATTRACTION / STEPS)
								..=EmbeddingParameters::MAX_ATTRACTION,
						)
						.step_by(EmbeddingParameters::MAX_ATTRACTION / STEPS)
						.text("attraction"),
					),
				)
				.union(
					ui.add(
						egui::Slider::new(
							&mut self.repulsion,
							(EmbeddingParameters::MAX_REPULSION / STEPS)
								..=EmbeddingParameters::MAX_REPULSION,
						)
						.step_by(EmbeddingParameters::MAX_REPULSION / STEPS)
						.text("repulsion"),
					),
				)
				.union(
					ui.add(
						egui::Slider::new(&mut self.repulsion_approximation, 0.0..=0.5)
							.step_by(0.5 / STEPS)
							.text("repulsion approximaton"),
					),
				)
				.union(
					ui.add(
						egui::Slider::new(
							&mut self.gravity,
							0.0..=EmbeddingParameters::MAX_GRAVITY,
						)
						.step_by(EmbeddingParameters::MAX_GRAVITY / STEPS)
						.text("gravity"),
					),
				);
			if self.statistic_updates_per_second == 0.0 {
				ui.label("Updates per second: paused");
			} else if self.statistic_updates_per_second.is_finite() {
				ui.label(format!(
					"Updates per second: {:.0}00",
					self.statistic_updates_per_second / 100.0
				));
			}
			resp
		})
		.inner
	}
}

pub struct GraphWidget {
	/// Linear zoom level:
	/// 1x => the graph fits within the area with some margin
	/// 10x => we are looking at a small part of the graph
	zoom: f32,
	pos: Vec2,
	active_node_and_pan: Option<(usize, PanState)>,
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum PanState {
	Centering,
	Centered,
	Off,
}
impl Default for GraphWidget {
	fn default() -> Self {
		Self {
			zoom: 1.0,
			pos: Vec2::ZERO,
			active_node_and_pan: None,
		}
	}
}

impl GraphWidget {
	pub fn deselect(&mut self) {
		self.active_node_and_pan = None;
	}

	pub fn active_node_id(&self) -> Option<usize> {
		self.active_node_and_pan.map(|(node, _)| node)
	}

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

				let scroll_area = egui::ScrollArea::both()
					.auto_shrink([false, false])
					.scroll_offset(self.pos)
					.show_viewport(ui, |ui, viewport| {
						draw_graph(
							ui,
							self.zoom,
							&mut self.active_node_and_pan,
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
				let background_drag = scroll_area.inner.drag_delta();

				let real_zoom_delta = if let (true, Some(hover_pos)) =
					(ui.ui_contains_pointer(), latest_pointer_pos)
				{
					// Scroll to zoom
					let new_zoom = (self.zoom * zoom_delta).max(1.0);
					let real_zoom_delta = new_zoom / self.zoom;
					self.zoom = new_zoom;

					let hover_pos = if self
						.active_node_and_pan
						.map_or(false, |(_, pan)| pan != PanState::Off)
					{
						scroll_area.inner_rect.size() / 2.0
					} else {
						hover_pos - scroll_area.inner_rect.min
					};
					let new_offset = (self.pos + hover_pos) * real_zoom_delta - hover_pos;
					self.pos = new_offset;
					real_zoom_delta
				} else {
					1.0
				};
				if background_drag != emath::Vec2::ZERO {
					// Drag to pan
					self.pos -= background_drag;
					if let Some((_, pan)) = self.active_node_and_pan.as_mut() {
						*pan = PanState::Off;
					}
				} else {
					// Automatic pan to active node
					if let Some((active, pan @ (PanState::Centering | PanState::Centered))) =
						&mut self.active_node_and_pan
					{
						let active_pos = translate_embed_to_scrollarea_pos(
							graph.node_positions[*active],
							graph,
							self.zoom,
							scroll_area.inner_rect.size(),
						)
						.to_vec2() - scroll_area.inner_rect.size() / 2.0;

						match pan {
							PanState::Centering => {
								const TAKE_FOCUS_SPEED: f32 = 0.5;
								self.pos = self.pos * (1.0 - TAKE_FOCUS_SPEED)
									+ active_pos * TAKE_FOCUS_SPEED;
								if (self.pos - active_pos).length() < self.pos.length() * 1e-6 {
									*pan = PanState::Centered;
								}
							}
							PanState::Centered => self.pos = active_pos,
							PanState::Off => unreachable!(),
						}
					}
				}

				//  Clip scrollarea position to content size
				self.pos = self.pos.max(emath::Vec2::ZERO).min(
					scroll_area.content_size * real_zoom_delta - scroll_area.inner_rect.size(),
				);
			});
	}
}

fn draw_graph(
	ui: &mut Ui,
	zoom_level: f32,
	active_node_and_pan: &mut Option<(usize, PanState)>,
	viewport: Rect,
	graph: &Graph2D,
) -> egui::Response {
	let offset = ui.cursor().left_top().to_vec2();
	let background = ui.allocate_rect(
		Rect::from_min_size(offset.to_pos2(), ui.available_size() * zoom_level),
		egui::Sense::drag(),
	);
	let style_widgets = &ui.visuals().widgets.clone();
	let style_selection = &ui.visuals().selection.clone();

	let scrollarea_node_pos: Vec<emath::Pos2> = graph
		.node_positions
		.iter()
		.map(|&p| translate_embed_to_scrollarea_pos(p, graph, zoom_level, viewport.size()))
		.collect();

	let (node_size, node_has_self_edge) = {
		let mut node_size = vec![std::f32::INFINITY; graph.node_positions.len()];
		let mut node_has_self_edge = vec![false; graph.node_positions.len()];
		for &(a, b) in &graph.edges {
			if a == b {
				node_has_self_edge[a] = true;
			} else {
				let d = scrollarea_node_pos[a].distance(scrollarea_node_pos[b]);
				node_size[a] = node_size[a].min(0.6 * d);
				node_size[b] = node_size[b].min(0.6 * d);
			}
		}
		let avg_size = node_size.iter().copied().sum::<f32>() / node_size.len() as f32;
		for size in &mut node_size {
			*size = size.clamp(avg_size / 3.0, avg_size * 2.0);
		}
		(node_size, node_has_self_edge)
	};

	let expanded_viewport = viewport.expand(
		node_size
			.iter()
			.copied()
			.reduce(|a, b| a.max(b))
			.unwrap_or(0.0)
			/ 2.0,
	);

	for (i, &pos) in scrollarea_node_pos.iter().enumerate() {
		if expanded_viewport.contains(pos) {
			let node = draw_node(
				ui,
				style_widgets,
				style_selection,
				pos,
				node_size[i],
				if node_has_self_edge[i] { "↺" } else { "A" },
				active_node_and_pan.map_or(false, |(node, _)| node == i),
				offset,
			);
			if node.drag_started() {
				*active_node_and_pan = Some((i, PanState::Centering));
			}
		}
	}

	for &(a, b) in &graph.edges {
		let origin = scrollarea_node_pos[a];
		let target = scrollarea_node_pos[b];
		if !viewport.intersects(Rect::from_two_pos(origin, target)) {
			continue;
		}
		edge_arrow(
			ui.painter(),
			origin + offset,
			target - origin,
			node_size[a],
			node_size[b],
			style_widgets.hovered.fg_stroke,
		);
	}
	background
}

fn draw_node(
	ui: &mut Ui,
	style_widgets: &egui::style::Widgets,
	style_selection: &egui::style::Selection,
	pos: egui::Pos2,
	node_width: f32,
	text: &str,
	selected: bool,
	offset: Vec2,
) -> Response {
	let rect =
		Rect::from_center_size(pos, egui::Vec2::new(node_width, node_width)).translate(offset);
	let resp = ui.allocate_rect(rect, Sense::click_and_drag());
	let lod_cutoff = ui.style().spacing.interact_size.y;
	let rounding = node_width / 5.0;
	let (bg_color, stroke) = if selected {
		(style_selection.bg_fill, style_selection.stroke)
	} else {
		(
			style_widgets.hovered.bg_fill,
			style_widgets.hovered.bg_stroke,
		)
	};

	ui.put(rect, move |ui: &mut Ui| {
		if node_width < lod_cutoff {
			ui.painter().rect_filled(rect, rounding, bg_color);
			ui.painter().rect_stroke(rect, rounding, stroke);
		} else {
			egui::Frame::none()
				.rounding(rounding)
				.fill(bg_color)
				.stroke(stroke)
				.show(ui, |ui| {
					ui.label(egui::RichText::new(text).small());
				});
		}
		resp
	})
}

fn edge_arrow(
	painter: &egui::Painter,
	mut origin: egui::Pos2,
	vec: egui::Vec2,
	node_size_origin: f32,
	node_size_target: f32,
	stroke: egui::Stroke,
) {
	let mut tip = origin + vec;
	let margin_origin = node_size_origin / 2.0 * vec / vec.abs().max_elem();
	let margin_tip = node_size_target / 2.0 * vec / vec.abs().max_elem();
	origin += margin_origin;
	tip -= margin_tip;

	let rot = emath::Rot2::from_angle(std::f32::consts::TAU / 10.0);
	let tip_length = ((tip - origin).length() / 4.0).min(node_size_target / 3.0);
	let dir = vec.normalized();
	painter.line_segment([origin, tip], stroke);
	painter.line_segment([tip, tip - tip_length * (rot * dir)], stroke);
	painter.line_segment(
		[tip, tip - tip_length * (rot.inverse() * dir)],
		stroke,
	);
}

fn translate_embed_to_scrollarea_pos(
	embed_space_pos: glam::DVec2,
	graph: &Graph2D,
	zoom_level: f32,
	viewport_size: Vec2,
) -> emath::Pos2 {
	let unit_square_pos =
		glam_to_emath((embed_space_pos - graph.min) / (graph.max - graph.min).max_element());
	let nozoom_viewport_margin = (viewport_size.min_elem() / 20.0).min(50.0);
	let nozoom_content_width = (viewport_size.min_elem() - 2.0 * nozoom_viewport_margin) * 0.9;
	let nozoom_offset = (viewport_size - emath::Vec2::splat(nozoom_content_width)) / 2.0;
	let nozoom_pos = nozoom_offset + unit_square_pos * nozoom_content_width;
	(nozoom_pos * zoom_level).to_pos2()
}

fn glam_to_emath(v: glam::DVec2) -> emath::Vec2 {
	emath::Vec2::from(<[f32; 2]>::from(v.as_vec2()))
}
