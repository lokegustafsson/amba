use std::{
	sync::{mpsc, Arc, Mutex, RwLock},
	thread,
	time::Instant,
};

use graphui::{
	EmbedderHasConverged, EmbeddingParameters, Graph2D, GraphWidget, LodText, NodeDrawingData,
};
use tracing_subscriber::{filter::targets::Targets, layer::Layer};

mod example_graph;

struct GraphTestGui {
	graph: Arc<RwLock<Graph2D>>,
	params: Arc<Mutex<EmbeddingParameters>>,
	notify_params_changed: mpsc::Sender<()>,
	graph_widget: GraphWidget,
}
impl eframe::App for GraphTestGui {
	fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
		egui::CentralPanel::default().show(ctx, |ui| {
			let params_widget = ui.add(&mut *self.params.lock().unwrap());
			if params_widget.changed() {
				self.notify_params_changed.send(()).unwrap();
			}
			self.graph_widget.show(
				ui,
				&self.graph.read().unwrap(),
				graphui::ColouringMode::AllGrey,
			);
		});
	}
}

fn main() {
	// Pinned version of winit crashes on wayland
	std::env::remove_var("WAYLAND_DISPLAY");

	tracing::subscriber::set_global_default(
		Targets::new()
			.with_target("eframe::native::run", tracing::Level::DEBUG)
			.with_target("egui_glow", tracing::Level::INFO)
			.with_default(tracing::Level::TRACE)
			.with_subscriber(
				tracing_subscriber::FmtSubscriber::builder()
					.with_max_level(tracing::Level::TRACE)
					.finish(),
			),
	)
	.expect("enabling global logger");

	let params = Arc::new(Mutex::new(EmbeddingParameters::default()));
	let graph = Arc::new(RwLock::new({
		let (node_count, edges) = example_graph::example_node_count_and_edges();
		Graph2D::new(
			(0..node_count)
				.map(|i| {
					let mut ret = LodText::new();
					ret.coarser(i.to_string());
					NodeDrawingData {
						state: 0,
						scc_group: 0,
						function: 0,
						lod_text: ret,
					}
				})
				.collect(),
			edges,
		)
	}));

	let worker_params = Arc::clone(&params);
	let worker_graph = Arc::clone(&graph);
	let (notify_params_changed_tx, notify_params_changed_rx) = mpsc::channel();

	let graph_widget = GraphWidget::default();

	eframe::run_native(
		"amba",
		eframe::NativeOptions {
			default_theme: eframe::Theme::Light,
			..Default::default()
		},
		Box::new(move |cc| {
			let ctx = cc.egui_ctx.clone();
			thread::spawn(move || loop {
				let mut working_copy = worker_graph.read().unwrap().clone();
				let params = *worker_params.lock().unwrap();

				let timer = Instant::now();
				working_copy.set_params(params);
				let converged = working_copy.run_layout_iterations(100);
				{
					let mut params = worker_params.lock().unwrap();
					if converged == EmbedderHasConverged::Yes {
						params.statistic_updates_per_second = 0.0;
						let _ = notify_params_changed_rx.recv();
					}
				}

				worker_params.lock().unwrap().statistic_updates_per_second =
					100.0 * timer.elapsed().as_secs_f64().recip();
				*worker_graph.write().unwrap() = working_copy;

				ctx.request_repaint();
			});
			Box::new(GraphTestGui {
				graph,
				params,
				graph_widget,
				notify_params_changed: notify_params_changed_tx,
			})
		}),
	)
	.unwrap();
}

#[cfg(test)]
mod test {
	use std::time::Duration;

	#[test]
	fn open_window() {
		use std::thread;

		thread::spawn(crate::main);
		// 10 seconds to let the OS find libraries, wait for X to be slow and so on
		thread::sleep(Duration::from_secs(10));
	}
}
