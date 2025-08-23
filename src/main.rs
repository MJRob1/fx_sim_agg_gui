//! # FX Simulator and Aggregator - fx_sim_agg_gui
//!
//! `fx_sim_agg_gui` simulates FX market data streams and aggregates them into a real-time book of buys and sells.
//!
//! - `main.rs` combines all the individual asynchronous market data streams from each liquidity provider into a single merged stream
//! that yields values in the order they arrive from the source market data streams
//! - `simulator.rs` generates simulated FX market data and sends the data as asynchronous market data streams
//! - `aggregator.rs` updates and aggregates the asynchronous data streams into a real-time FX book of buys and sells
//! - `lib.rs` various utilities used by the other modules
use std::process::exit;
//use log::{debug, error, info, trace, warn};
use egui::Vec2;
use log::error;

fn main() {
    // start log4rs logging framework
    if let Err(e) = log4rs::init_file("logging_config.yaml", Default::default()) {
        eprintln!("error initialising log4rs - {e}");
        exit(1);
    }

    // let win_option = eframe::NativeOptions::default();
    let mut fx_viewer_app = fx_sim_agg_gui::FxViewerApp::default();
    let win_option = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size(Vec2::new(510., 450.)),
        ..Default::default()
    };
    if let Err(e) = eframe::run_native(
        "USD/EUR Aggregated Book",
        win_option,
        Box::new(|cc| Ok(Box::new(fx_viewer_app.init(cc)))),
    ) {
        error!("error starting eframe - {e}");
        exit(1);
    }
}
