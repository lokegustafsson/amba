use std::{
	sync::{Arc, Mutex, RwLock},
	thread,
};

use graphui::{EmbeddingParameters, Graph2D, GraphWidget};

mod example_graph;

struct GraphTestGui {
	graph: Arc<RwLock<Graph2D>>,
	params: Arc<Mutex<EmbeddingParameters>>,
	graph_widget: GraphWidget,
}
impl eframe::App for GraphTestGui {
	fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
		egui::CentralPanel::default().show(ctx, |ui| {
			ui.add(&mut *self.params.lock().unwrap());
			self.graph_widget.show(ui, &self.graph.read().unwrap());
		});
	}
}

fn main() {
	let params = Arc::new(Mutex::new(EmbeddingParameters::default()));
	let graph = Arc::new(RwLock::new(Graph2D::new(
		example_graph::example_graph(),
		*params.lock().unwrap(),
	)));

	let worker_params = Arc::clone(&params);
	let worker_graph = Arc::clone(&graph);

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
				working_copy.run_layout_iterations(100, params);
				*worker_graph.write().unwrap() = working_copy;
				ctx.request_repaint();
			});
			Box::new(GraphTestGui {
				graph,
				params,
				graph_widget,
			})
		}),
	)
	.unwrap()
}
