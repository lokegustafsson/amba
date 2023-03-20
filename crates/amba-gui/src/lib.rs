mod app;
use app::NodeGraphExample;

pub fn main() {
	use eframe::egui::Visuals;

	eframe::run_native(
		"Egui node graph example",
		eframe::NativeOptions::default(),
		Box::new(|cc| {
			cc.egui_ctx.set_visuals(Visuals::dark());
			Box::new(NodeGraphExample::default())
		}),
	);
}
