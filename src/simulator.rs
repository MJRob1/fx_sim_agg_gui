use core::f64;
//use log::{debug, error, info, trace, warn};
use log::{error, info};
use std::fs;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::{spawn, sync::mpsc::unbounded_channel, time::sleep};
use tokio_stream::{Stream, StreamMap, wrappers::UnboundedReceiverStream};

use crate::{AppError, get_str_field};

#[derive(Debug)]
pub struct Config {
    pub liquidity_provider: String,
    pub currency_pair: String,
    pub buy_price: f64,
    pub spread: f64,
    pub three_mill_markup: f64,
    pub five_mill_markup: f64,
    pub run_iterations: i32,
}

pub fn get_configs(configs: &mut Vec<Config>) -> Result<(), AppError> {
    let parameters = read_config_file("resources/config.txt")?;
    let mut index = 0;
    for i in &parameters {
        // ignore header line in config file
        if index > 0 {
            let mut fx_params = i.split(",");
            // let mut fx_params = fx_sim_agg_gui::get_params(i, 7)?;
            let liquidity_provider = get_str_field(fx_params.next())?;
            let currency_pair = get_str_field(fx_params.next())?;
            let buy_price: f64 = fx_params.next().unwrap_or("").trim().parse()?;
            let mut spread: f64 = fx_params.next().unwrap_or("").trim().parse()?;
            spread = spread / 10000.0;
            let mut three_mill_markup: f64 = fx_params.next().unwrap_or("").trim().parse()?;
            three_mill_markup = three_mill_markup / 10000.0;
            let mut five_mill_markup: f64 = fx_params.next().unwrap_or("").trim().parse()?;
            five_mill_markup = five_mill_markup / 10000.0;
            let run_iterations: i32 = fx_params.next().unwrap_or("").trim().parse()?;

            let config = Config {
                liquidity_provider: String::from(liquidity_provider),
                currency_pair: String::from(currency_pair),
                buy_price,
                spread,
                three_mill_markup,
                five_mill_markup,
                run_iterations,
            };

            configs.push(config);
        }
        index += 1;
    }

    Ok(())
}

fn read_config_file<P: AsRef<Path>>(file_path: P) -> Result<Vec<String>, AppError> {
    let contents: String = fs::read_to_string(file_path)?;
    let results = contents.lines().map(String::from).collect();
    Ok(results)
}

pub fn get_marketdata(config: &Config) -> impl Stream<Item = String> {
    // For this liqudity provider in config, create the new market data values
    // and send them asynchronously (don't block and wait) every random 1000-5000 milliseconds
    let (tx, rx) = unbounded_channel();

    // async block may outlive the current function, and the config reference only lives for the current function
    // async blocks are not executed immediately and must either take a reference or ownership of outside variables they use
    // can't take a reference of config values because config is a shared reference, hence left to take ownership of new variables
    // from config and use them in the async block below. Also can't use lifetimes because Stream returned from the function can outlive the function
    let mut buy_price = config.buy_price;
    let spread = config.spread;
    let three_mill_markup = config.three_mill_markup;
    let five_mill_markup = config.five_mill_markup;
    let number_iterations = config.run_iterations + 1;
    let liquidity_provider = config.liquidity_provider.clone();
    let currency_pair = config.currency_pair.clone();

    spawn(async move {
        // spawn a task to handle the async sleep calls
        // async returns a future rather than blocking current thread
        // move is required to move tx into the async block so it gets ownership and
        // tx closes after last message is sent
        for _number in 1..number_iterations {
            let random_sleep = rand::random_range(1000..5000);
            //   println!("random sleep is {random_sleep}");
            // await polls the future until future returns Ready.
            // If future still pending then control is handed to the runtime
            sleep(Duration::from_millis(random_sleep)).await;
            // now future has returned ready state and so code below is now executed

            // randomly determine whether this is a price rise or fall
            // let pip_change: f64 = rand::random_range(1.0..5.0) / 10000.0;
            let pip_change: f64 = rand::random_range(0.0..2.0) / 10000.0;

            // let's say it is a bull market and prices are trending up
            buy_price = ((buy_price + pip_change) * 10000.0).round() / 10000.0;

            let sell_price = ((buy_price + spread) * 10000.0).round() / 10000.0;
            let three_mill_buy_price =
                ((buy_price + three_mill_markup) * 10000.0).round() / 10000.0;
            let three_mill_sell_price =
                ((sell_price - three_mill_markup) * 10000.0).round() / 10000.0;
            let five_mill_buy_price = ((buy_price + five_mill_markup) * 10000.0).round() / 10000.0;
            let five_mill_sell_price =
                ((sell_price - five_mill_markup) * 10000.0).round() / 10000.0;
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();

            let marketdata = format!(
                "{} | {} | {} | {} | {} | {} | {} | {} | {}",
                liquidity_provider,
                currency_pair,
                buy_price,
                sell_price,
                three_mill_buy_price,
                three_mill_sell_price,
                five_mill_buy_price,
                five_mill_sell_price,
                timestamp
            );

            if let Err(send_error) = tx.send(format!("{marketdata}")) {
                error!("could not send message {marketdata}: {send_error}");
                break;
            };
        }

        // number of iterations done so exit the program
        info!("{} stream completed", liquidity_provider);
    });

    UnboundedReceiverStream::new(rx)
}

pub fn start_streams(config: &Vec<Config>) -> StreamMap<i32, impl Stream<Item = String>> {
    let mut index = 0;
    let mut map = StreamMap::new();
    // start a market data simulated stream for each config (liquidity provider) value
    // Combine all individual market data streams from each liquidity provider into a single merged stream map
    for i in config {
        let marketdata = get_marketdata(i);

        map.insert(index, marketdata);
        index += 1;
    }
    map
}
