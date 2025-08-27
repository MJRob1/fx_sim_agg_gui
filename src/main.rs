//! # FX Simulator and Aggregator - fx_sim_agg_gui
//!
//! `fx_sim_agg_gui` simulates FX market data streams and aggregates them into a real-time book of buys and sells.
//! A separate thread renders the FX updates in real-time to a GUI.
//!
//! - `main.rs`  Defines and initiates the UI runtime (which in turn intiates the asynchronous fx simulation and aggregation runtime). Also initiates log4rs logging framework
//! - `simulator.rs` generates simulated FX market data and sends the data as asynchronous market data streams
//! - `aggregator.rs` updates and aggregates the asynchronous data streams into a real-time FX book of buys and sells
//! - `lib.rs` Includes the thread which combines all the individual asynchronous market data streams from each liquidity provider into a single merged stream
//! that yields values in the order they arrive from the source market data streams. Also incudes the FxViewerApp structure which initiates and updates the GUI.
//! Various utilities used by the other modules are also in this library.
//! - `gui.rs` Contains the definition of the GUI components and how to render them.
use std::process::exit;
//use log::{debug, error, info, trace, warn};
use egui::Vec2;
use fx_sim_agg_gui::FxViewerApp;
use log::error;

fn main() {
    // start log4rs logging framework
    if let Err(e) = log4rs::init_file("logging_config.yaml", Default::default()) {
        eprintln!("error initialising log4rs - {e}");
        exit(1);
    }

    let mut fx_viewer_app = FxViewerApp::default();
    let win_option = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size(Vec2::new(590., 350.)),
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
