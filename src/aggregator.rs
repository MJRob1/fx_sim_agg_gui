//! # FX Simulator and Aggregator - fx_sim_agg_gui
//!
//! `aggregator.rs` aggregates simulated FX market data streams into a real-time book of buys and sells.
use crate::simulator::Config;
use crate::{AppError, get_params, get_str_field};
extern crate chrono;
use chrono::Utc;
use chrono::prelude::DateTime;
use core::f64;
//use log::{debug, error, info, trace, warn};
use log::{error, info};
use std::cmp::Ordering;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub struct FxAggBookEntry {
    pub lp_vol: Vec<(String, i32)>,
    pub volume: i32,
    pub price: f64,
    pub side: String,
}

impl PartialEq for FxAggBookEntry {
    fn eq(&self, other: &Self) -> bool {
        self.price == other.price
    }
}

impl PartialOrd for FxAggBookEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.price.partial_cmp(&other.price)
    }
}
#[derive(Debug, Default)]
pub struct FxBook {
    pub currency_pair: String,
    pub buy_book: Vec<FxAggBookEntry>,
    pub sell_book: Vec<FxAggBookEntry>,
    pub timestamp: u64,
}

impl FxBook {
    pub fn update(&mut self, market_data: String) -> Result<(), AppError> {
        // add fxbook entries for all current market data in the order of
        // 1M buy, 1M sell, 3M buy, 3M sell, 5M buy, 5M sell
        add_market_data(self, market_data)?;
        sort_books(self);
        match check_books_crossed(self) {
            Some(index) => {
                info!(
                    "books crossed at sell book index {} with sell price {}",
                    index.0, index.1
                );
                correct_crossed_books(self, index)?;
            }
            None => (),
        }
        maintain_min_spread(self);
        Ok(())
    }
    pub fn new(config: &Vec<Config>) -> Self {
        // create a new FxBook with empty buy and sell books
        // and a timestamp of current time
        // using first config entry to get currency pair
        let currency_pair = config[0].currency_pair.clone();
        let buy_book: Vec<FxAggBookEntry> = Vec::new();
        let sell_book: Vec<FxAggBookEntry> = Vec::new();
        //need to catch this possible panic on unwrap when converting u126 to u64
        let timestamp: u64 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            .try_into()
            .unwrap();

        FxBook {
            currency_pair,
            buy_book,
            sell_book,
            timestamp,
        }
    }
}

fn correct_crossed_books(fx_book: &mut FxBook, index: (usize, f64)) -> Result<(), AppError> {
    // when books have crossed then need to remove all entries above the cross price from the
    // top of the book that has the highest number of entries
    if fx_book.buy_book.len() > fx_book.sell_book.len() {
        let fx_book_side = get_book_side(fx_book, "Buy");
        info!("buy book longer than sell book when books have crossed");
        if let Some(position) = find_buy_index_when_crossed(fx_book_side, index.1) {
            // remove all buy entries >= new sell price
            info!(
                "removing all buy entries from top of book to index {}",
                position
            );
            remove_range_entries_from_top(fx_book_side, position, "Buy");
        }
    } else {
        // remove all sell entries <= new buy price
        let fx_book_side = get_book_side(fx_book, "Sell");
        info!(
            "removing all sell entries from top of book to index {}",
            index.0
        );
        remove_range_entries_from_top(fx_book_side, index.0, "Sell");
    }

    Ok(())
}
fn add_market_data(fx_book: &mut FxBook, market_data: String) -> Result<(), AppError> {
    let mut vol_prices_vec: Vec<(i32, f64, String)> = Vec::new();

    let mut market_data_params = get_params(&market_data, 9)?;
    let liquidity_provider = get_str_field(market_data_params.next())?;
    let _currency_pair = get_str_field(market_data_params.next())?;
    let one_mill_buy_price: f64 = market_data_params.next().unwrap_or("").trim().parse()?;
    vol_prices_vec.push((1, one_mill_buy_price, String::from("Buy")));
    let one_mill_sell_price: f64 = market_data_params.next().unwrap_or("").trim().parse()?;
    vol_prices_vec.push((1, one_mill_sell_price, String::from("Sell")));
    let three_mill_buy_price: f64 = market_data_params.next().unwrap_or("").trim().parse()?;
    vol_prices_vec.push((3, three_mill_buy_price, String::from("Buy")));
    let three_mill_sell_price: f64 = market_data_params.next().unwrap_or("").trim().parse()?;
    vol_prices_vec.push((3, three_mill_sell_price, String::from("Sell")));
    let five_mill_buy_price: f64 = market_data_params.next().unwrap_or("").trim().parse()?;
    vol_prices_vec.push((5, five_mill_buy_price, String::from("Buy")));
    let five_mill_sell_price: f64 = market_data_params.next().unwrap_or("").trim().parse()?;
    vol_prices_vec.push((5, five_mill_sell_price, String::from("Sell")));
    let timestamp: u64 = market_data_params.next().unwrap_or("").trim().parse()?;

    fx_book.timestamp = timestamp;

    let mut i = 0;
    for val in vol_prices_vec {
        if i % 2 == 0 {
            //remove expired quotes before adding any new quotes
            let fx_book_side = get_book_side(fx_book, "Buy");
            // match check_expired_quotes(fx_book, liquidity_provider, "Buy", val.0) {
            match check_expired_quotes(fx_book_side, liquidity_provider, val.0) {
                Some(entry_to_remove) => remove_single_entry(fx_book, "Buy", entry_to_remove),
                None => (),
            }
            add_agg_book_entry(fx_book, liquidity_provider, val.0, val.1, "Buy");
        } else {
            let fx_book_side = get_book_side(fx_book, "Sell");
            match check_expired_quotes(fx_book_side, liquidity_provider, val.0) {
                //  match check_expired_quotes(fx_book, liquidity_provider, "Sell", val.0) {
                Some(entry_to_remove) => remove_single_entry(fx_book, "Sell", entry_to_remove),
                None => (),
            }
            add_agg_book_entry(fx_book, liquidity_provider, val.0, val.1, "Sell");
        }
        i += 1;
    }

    Ok(())
}

pub fn check_expired_quotes(
    // fx_book: &mut FxBook,
    fx_book_side: &mut Vec<FxAggBookEntry>,
    liquidity_provider: &str,
    // side: &str,
    volume: i32,
) -> Option<usize> {
    // check to see if there is an expired quote from this liquidity provider
    let mut index = 0;
    let mut index_to_remove: usize = 0;
    let mut remove_entry = false;
    for entry in fx_book_side {
        let mut total_volume = 0;
        let lp_vol_vec: &mut Vec<(String, i32)> = &mut entry.lp_vol;
        // remove expired quote
        lp_vol_vec.retain(|lp_vol| {
            (lp_vol.0 != liquidity_provider)
                || ((lp_vol.0 == liquidity_provider) && (lp_vol.1 != volume))
        });
        // check to see if removing expired quote has left behind an fxbook entry with an
        // empty liquidity provider and volume pair vector. Return index of this entry so
        // remove_single_entry function can remove this entry
        if lp_vol_vec.len() == 0 {
            index_to_remove = index;
            remove_entry = true;
        }
        // need to re-sum the total volumes here in case an expired quote has been removed
        for val in lp_vol_vec {
            total_volume += val.1;
        }
        entry.volume = total_volume;
        index += 1;
    }

    if remove_entry {
        return Some(index_to_remove);
    } else {
        return None;
    }
}

pub fn add_agg_book_entry(
    fx_book: &mut FxBook,
    liquidity_provider: &str,
    volume: i32,
    price: f64,
    side: &str,
) {
    let mut lp_vol_vec: Vec<(String, i32)> = Vec::new();
    lp_vol_vec.push((String::from(liquidity_provider), volume));

    // if first entry then just add it to book
    // and using fact that first entry is always a Buy in current config
    if fx_book.buy_book.len() == 0 && side == "Buy" {
        let new_agg_book_entry = FxAggBookEntry {
            lp_vol: lp_vol_vec,
            volume,
            price,
            side: String::from(side),
        };
        fx_book.buy_book.push(new_agg_book_entry);
        println!("{fx_book:?}");
        return;
    } else if fx_book.buy_book.len() == 0 && fx_book.sell_book.len() == 0 {
        error!("first entry should not be on sell side in current configuration");
        return;
    } else {
        let fx_book_side = get_book_side(fx_book, side);

        //search to see if current price already in aggregated book
        for entry in fx_book_side {
            if entry.price == price {
                let lp_tup = (String::from(liquidity_provider), volume);
                entry.lp_vol.push(lp_tup);
                entry.volume += volume;
                return;
            }
        }

        // this is new entry
        let new_agg_book_entry = FxAggBookEntry {
            lp_vol: lp_vol_vec,
            volume,
            price,
            side: String::from(side),
        };
        let fx_book_side = get_book_side(fx_book, side);
        fx_book_side.push(new_agg_book_entry);
        return;
    }
}

fn sort_books(fx_book: &mut FxBook) {
    let fx_buy_book = get_book_side(fx_book, "Buy");
    sort_buy_book(fx_buy_book);
    let fx_sell_book = get_book_side(fx_book, "Sell");
    sort_sell_book(fx_sell_book);

    /*   fx_book
    .sell_book
    .sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap()); */
}

pub fn sort_buy_book(fx_buy_book: &mut Vec<FxAggBookEntry>) {
    // need to do a reverse sort on price for buy side
    fx_buy_book.sort_by(|a, b| match a.price.partial_cmp(&b.price).unwrap() {
        Ordering::Less => Ordering::Greater,
        Ordering::Equal => Ordering::Equal,
        Ordering::Greater => Ordering::Less,
    });
}

pub fn sort_sell_book(fx_sell_book: &mut Vec<FxAggBookEntry>) {
    fx_sell_book.sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap());
}

pub fn find_buy_index_when_crossed(
    fx_buy_book: &mut Vec<FxAggBookEntry>,
    sell_price: f64,
) -> Option<usize> {
    // when books have crossed and buy book is longer than sell book
    // then need to find where buy price crosses on sell side and remove
    // all buy entries >= new sell price

    for i in (0..fx_buy_book.len()).rev() {
        if fx_buy_book[i].price >= sell_price {
            return Some(i);
        }
    }
    return None;
}

pub fn check_books_crossed(fx_book: &mut FxBook) -> Option<(usize, f64)> {
    let top_of_buy_book_price = fx_book.buy_book[0].price;
    let fx_book_side = get_book_side(fx_book, "Sell");

    // if buy book top of book price >= any fx_book.sell_book price then books have crossed
    // so find where buy price crosses on sell side and remove all sell entries <= new buy price
    for i in (0..fx_book_side.len()).rev() {
        if top_of_buy_book_price >= fx_book_side[i].price {
            return Some((i, fx_book_side[i].price));
        }
    }
    return None;
}

pub fn maintain_min_spread(fx_book: &mut FxBook) {
    // if spread is less than 6 pips (arbitrary) then delete top of book entries
    // until get this minimum spread

    while fx_book.sell_book[0].price - fx_book.buy_book[0].price <= 0.0006 {
        if fx_book.buy_book.len() >= fx_book.sell_book.len() {
            // remove top entry from buy side
            info!("removing top of buy book to maintain spread");
            remove_single_entry(fx_book, "Buy", 0);
        } else {
            info!("removing top of sell book to maintain spread");
            remove_single_entry(fx_book, "Sell", 0)
        }
    }
}

pub fn remove_range_entries_from_top(
    fx_book_side: &mut Vec<FxAggBookEntry>,
    index: usize,
    side: &str,
) {
    //  let fx_book_side = get_book_side(fx_book, side);
    for i in 0..index + 1 {
        // because of removal of [0] entry then entry to remove is always the top one [0]
        fx_book_side.remove(0);
        info!("removing entry {} from {} book", i, side);
    }
}

fn remove_single_entry(fx_book: &mut FxBook, side: &str, index_to_remove: usize) {
    // removing an expired quote can leave behind an fxbook entry with an empty
    // liquidity provider and volume vector. This funtion removes this hanging entry

    let fx_book_side = get_book_side(fx_book, side);
    fx_book_side.remove(index_to_remove);
}

pub fn get_book_side<'a>(fx_book: &'a mut FxBook, side: &str) -> &'a mut Vec<FxAggBookEntry> {
    // Because fx_book is the argument that contains the returned vector of book entries
    // then this fx_book argument is the argument that must be connected to the return
    // value using the lifetime syntax
    if String::from(side) == String::from("Buy") {
        &mut fx_book.buy_book
    } else {
        &mut fx_book.sell_book
    }
}

pub fn print_fxbook_as_ladder(fx_book: &mut FxBook) {
    let d = UNIX_EPOCH + Duration::from_nanos(fx_book.timestamp);
    let datetime = DateTime::<Utc>::from(d);

    println!(
        "\nCurrent state of FX Book for {} at timestamp {}:\n",
        fx_book.currency_pair,
        datetime.format("%Y-%m-%d %H:%M:%S.%f").to_string()
    );
    println!("Side\t Price\t Volume(M)\t (Liquidity Providers : Volumes(M))");
    println!("===================================================================");
    print_sell_side(fx_book);
    println!("<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>");
    print_buy_side(fx_book);
}

fn print_buy_side(fx_book: &mut FxBook) {
    let fx_book_side = get_book_side(fx_book, "Buy");
    for entry in fx_book_side {
        print!("{}:\t {}\t   {}", entry.side, entry.price, entry.volume);
        let lp_vol_vec: &mut Vec<(String, i32)> = &mut entry.lp_vol;
        let len = lp_vol_vec.len() - 1;
        let mut index = 0;
        for val in lp_vol_vec {
            if index == 0 && len == 0 {
                let lp_vol = format!("\t\t ({}: {})", val.0, val.1);
                print!("{}", lp_vol);
            } else if index == 0 {
                let lp_vol = format!("\t\t ({}: {},", val.0, val.1);
                print!("{}", lp_vol);
            } else if index == len {
                let lp_vol = format!(" {}: {})", val.0, val.1);
                print!("{}", lp_vol);
            } else {
                let lp_vol = format!(" {}: {},", val.0, val.1);
                print!("{}", lp_vol);
            }
            index += 1;
        }
        print!("\n");
    }
}

fn print_sell_side(fx_book: &mut FxBook) {
    let fx_book_side = get_book_side(fx_book, "Sell");

    for i in (0..fx_book_side.len()).rev() {
        print!(
            "{}:\t {}\t   {}",
            fx_book_side[i].side, fx_book_side[i].price, fx_book_side[i].volume
        );
        let lp_vol_vec: &mut Vec<(String, i32)> = &mut fx_book_side[i].lp_vol;
        let len = lp_vol_vec.len() - 1;
        let mut index = 0;
        for val in lp_vol_vec {
            if index == 0 && len == 0 {
                let lp_vol = format!("\t\t ({}: {})", val.0, val.1);
                print!("{}", lp_vol);
            } else if index == 0 {
                let lp_vol = format!("\t\t ({}: {},", val.0, val.1);
                print!("{}", lp_vol);
            } else if index == len {
                let lp_vol = format!(" {}: {})", val.0, val.1);
                print!("{}", lp_vol);
            } else {
                let lp_vol = format!(" {}: {},", val.0, val.1);
                print!("{}", lp_vol);
            }
            index += 1;
        }
        print!("\n");
    }
}
