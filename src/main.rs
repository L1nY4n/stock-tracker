#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::{egui::ViewportBuilder, run_native, NativeOptions};
use stock_tracker::ui::StockTrackerApp;
use tracing_subscriber;

fn main() {
    tracing_subscriber::fmt()
        .with_file(true)
        .with_line_number(true)
        .init();
    let viewport = ViewportBuilder::default()
        .with_decorations(false)
        .with_inner_size((220.0, 200.0))
        .with_transparent(true)
        .with_drag_and_drop(true);

    let native_options = NativeOptions {
        viewport: viewport,
        ..Default::default()
    };

    // let native_options = NativeOptions {
    //     initial_window_size: Some(Vec2 { x: 220.0, y: 200.0 }),
    //     transparent: true,
    //     decorated: false,
    //     always_on_top: false,
    //    Default::default()
    // };

    let _ = run_native(
        "St Tracker",
        native_options,
        Box::new(|cc| Ok(Box::new(StockTrackerApp::new(cc)))),
    );
}
