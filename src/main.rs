#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::{epaint::Vec2, run_native, NativeOptions};
use stock_tracker::ui::StockTrackerApp;
use tracing_subscriber;

fn main() {
    tracing_subscriber::fmt()
        .with_file(true)
        .with_line_number(true)
        .init();

    let native_options = NativeOptions {
        initial_window_size: Some(Vec2 { x: 220.0, y: 200.0 }),
        transparent: true,
        decorated: false,
        always_on_top: true,
        ..NativeOptions::default()
    };
    run_native(
        "S tracker",
        native_options,
        Box::new(|cc| Box::new(StockTrackerApp::new(cc))),
    )
}
