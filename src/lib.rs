//! # FX Simulator and Aggregator - fx_sim_agg
//!
//! `fx_sim_agg` simulates FX market data streams and aggregates them into a real-time book of buys and sells.
//!
//! - `main.rs` combines all the individual asynchronous market data streams from each liquidity provider into a single merged stream
//! that yields values in the order they arrive from the source market data streams
//! - `simulator.rs` generates simulated FX market data and sends the data as asynchronous market data streams
//! - `aggregator.rs` updates and aggregates the asynchronous data streams into a real-time FX book of buys and sells
mod aggregator;
mod gui;
mod simulator;
use eframe::egui;
use egui::Context;
use log::error;
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fs::File;
use std::io;
use std::io::BufWriter;
use std::io::prelude::*;
use std::num::ParseFloatError;
use std::num::ParseIntError;
use std::path::Path;
use std::process::exit;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use tokio::runtime::Runtime;
use tokio_stream::StreamExt;

#[derive(Debug)]
#[non_exhaustive]
pub enum AppError {
    NumParams,
    IsEmpty,
    ParseFloat(ParseFloatError),
    ParseInt(ParseIntError),
    Io(io::Error),
}

impl From<ParseFloatError> for AppError {
    fn from(error: ParseFloatError) -> Self {
        Self::ParseFloat(error)
    }
}

impl From<ParseIntError> for AppError {
    fn from(error: ParseIntError) -> Self {
        Self::ParseInt(error)
    }
}

impl From<io::Error> for AppError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::IsEmpty => f.write_str("empty data field"),
            Self::NumParams => f.write_str("missing market data fields"),
            Self::ParseFloat(e) => Display::fmt(e, f),
            Self::ParseInt(e) => Display::fmt(e, f),
            Self::Io(e) => Display::fmt(e, f),
        }
    }
}

impl std::error::Error for AppError {}

pub fn run<F: Future>(future: F) -> F::Output {
    let rt = Runtime::new().unwrap();
    rt.block_on(future)
}

#[derive(Default, Debug)]
pub struct FxViewerApp {
    pub fx_book_mutex: Arc<Mutex<aggregator::FxBook>>,
    // pub fx_book: FxBook,
}

impl FxViewerApp {
    pub fn init(&mut self, cc: &eframe::CreationContext<'_>) -> Self {
        let ctx = cc.egui_ctx.clone();
        let (ctx_tx, ctx_rx) = mpsc::channel();
        // Create "FIX" log file
        let writer = match create_log_file("logs/fix.log") {
            Ok(writer) => writer,
            Err(e) => {
                error!("problem creating log file - {e}");
                exit(1);
            }
        };
        // read config file to get configs for each liquidity provider source
        let mut configs: Vec<simulator::Config> = Vec::new();
        if let Err(e) = simulator::get_configs(&mut configs) {
            error!("config input file not processed - {e}");
            exit(1);
        }
        // Create aggregated FX Book
        let fx_book = aggregator::FxBook::new(&configs);

        let fx_book_mutex = Arc::new(Mutex::new(fx_book));
        let fx_book_mutex_ui_clone = Arc::clone(&fx_book_mutex);
        let fx_book_mutex_fx_clone = Arc::clone(&fx_book_mutex);
        thread::spawn(move || {
            // start fx thread
            let rec_ctx: Context = ctx_rx.recv().unwrap();

            run_async_fx_sim_agg(rec_ctx, writer, fx_book_mutex_fx_clone, &configs);
            /*
            run(async {
                /*  async returns a future rather than blocking current thread
                run() starts a runtime and hands the future to the runtime all the code - the entire program
                is the signature future argument of run! Note: everything inside the async code avoids blocking
                but any code outside run will block on the run function returning */

                // Combine all individual market data streams from each liquidity provider into a single merged stream
                // that yields values in the order they arrive from the source market data streams
                let mut merged_streams_map = simulator::start_streams(&configs);

                while let Some(val) = merged_streams_map.next().await {
                    // await polls the future until future returns Ready.
                    // If future still pending then control is handed to the runtime
                    let (_key, market_data) = val;

                    // write market data to a "FIX" log
                    if let Err(e) = write_to_fix_log(&mut writer, &market_data) {
                        error!("problem writing to FIX log - {e}");
                    }

                    // Update the Fx Book with the new market data
                    let mut fx_book = fx_book_mutex_fx_clone.lock().unwrap();
                    if let Err(e) = fx_book.update(market_data) {
                        error!("market data not processed - {e}");
                    } else {
                        // currently working on real-time GUI rather than print!!
                        aggregator::print_fxbook_as_ladder(&mut fx_book);
                    }
                    // update GUI
                    rec_ctx.request_repaint();
                }
            });  */
        }); // end of fx thread  - mutex lock released here

        if let Err(e) = ctx_tx.send(ctx) {
            error!("error sending from ctx channel - {e}");
            exit(1);
        }

        Self {
            fx_book_mutex: fx_book_mutex_ui_clone,
        }
    } // mutex lock released here
}

impl eframe::App for FxViewerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        gui::render_top_panel(ctx);
        gui::render_fx_book(self, ctx);
    }
}

pub fn run_async_fx_sim_agg(
    rec_ctx: Context,
    mut writer: BufWriter<File>,
    fx_book_mutex_fx_clone: Arc<Mutex<aggregator::FxBook>>,
    configs: &Vec<simulator::Config>,
) {
    run(async {
        /*  async returns a future rather than blocking current thread
        run() starts a runtime and hands the future to the runtime all the code - the entire program
        is the signature future argument of run! Note: everything inside the async code avoids blocking
        but any code outside run will block on the run function returning */

        // Combine all individual market data streams from each liquidity provider into a single merged stream
        // that yields values in the order they arrive from the source market data streams
        let mut merged_streams_map = simulator::start_streams(configs);

        while let Some(val) = merged_streams_map.next().await {
            // await polls the future until future returns Ready.
            // If future still pending then control is handed to the runtime
            let (_key, market_data) = val;

            // write market data to a "FIX" log
            if let Err(e) = write_to_fix_log(&mut writer, &market_data) {
                error!("problem writing to FIX log - {e}");
            }

            // Update the Fx Book with the new market data
            let mut fx_book = fx_book_mutex_fx_clone.lock().unwrap();
            if let Err(e) = fx_book.update(market_data) {
                error!("market data not processed - {e}");
            } else {
                // currently working on real-time GUI rather than print!!
                aggregator::print_fxbook_as_ladder(&mut fx_book);
            }
            // update GUI
            rec_ctx.request_repaint();
        }
    });
}

pub fn create_log_file(file_path: &str) -> Result<BufWriter<File>, AppError> {
    let path = Path::new(file_path);

    // Open a file in write-only mode, returns `io::Result<File>`
    let file = File::create(&path)?;

    Ok(BufWriter::new(file))
}

pub fn write_to_fix_log(
    writer: &mut BufWriter<File>,
    market_data: &String,
) -> Result<(), AppError> {
    writeln!(writer, "{}", market_data)?;
    Ok(())
}

pub fn get_params(data: &str, number: usize) -> Result<std::str::Split<'_, &str>, AppError> {
    let value = data.split("|");
    if value.clone().count() < number {
        return Err(AppError::NumParams);
    } else {
        Ok(data.split("|"))
    }
}

pub fn get_str_field(field: Option<&str>) -> Result<&str, AppError> {
    let value = field.unwrap_or("");
    if value.trim().is_empty() {
        return Err(AppError::IsEmpty);
    } else {
        Ok(value.trim())
    }
}
