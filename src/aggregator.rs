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
    // when books have crossed then need to remove all entries from the
    // top of the book that has the lower number of entries
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
        let fx_book_side = get_book_side(fx_book, "Sell");
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

fn check_expired_quotes(
    // fx_book: &mut FxBook,
    fx_book_side: &mut Vec<FxAggBookEntry>,
    liquidity_provider: &str,
    // side: &str,
    volume: i32,
) -> Option<usize> {
    // compiler does not allow you to use fx_book.buy/sell_side as reference but
    // does let you create this new local reference from within that reference!
    // But need to use lifetime in get_book_side function to guarantee that fxbook
    // reference outlives this local reference
    //  let fx_book_side = get_book_side(fx_book, side);
    let mut index = 0;
    let mut index_to_remove: usize = 0;
    let mut remove_entry = false;
    // let mut total_volume = 0;
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

fn add_agg_book_entry(
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

fn sort_buy_book(fx_buy_book: &mut Vec<FxAggBookEntry>) {
    // need to do a reverse sort on price for buy side
    fx_buy_book.sort_by(|a, b| match a.price.partial_cmp(&b.price).unwrap() {
        Ordering::Less => Ordering::Greater,
        Ordering::Equal => Ordering::Equal,
        Ordering::Greater => Ordering::Less,
    });
}

fn sort_sell_book(fx_sell_book: &mut Vec<FxAggBookEntry>) {
    fx_sell_book.sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap());
}

fn find_buy_index_when_crossed(
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

fn check_books_crossed(fx_book: &mut FxBook) -> Option<(usize, f64)> {
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

fn maintain_min_spread(fx_book: &mut FxBook) {
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

fn remove_range_entries_from_top(fx_book_side: &mut Vec<FxAggBookEntry>, index: usize, side: &str) {
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

fn get_book_side<'a>(fx_book: &'a mut FxBook, side: &str) -> &'a mut Vec<FxAggBookEntry> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sort_by_price_reverse() {
        let mut fx_buy_book: Vec<FxAggBookEntry> = vec![
            FxAggBookEntry {
                lp_vol: vec![
                    (String::from("MS "), 1),
                    (String::from("UBS "), 5),
                    (String::from("CITI "), 3),
                    (String::from("BARX "), 3),
                ],
                volume: 12,
                price: 1.5555,
                side: String::from("Buy"),
            },
            FxAggBookEntry {
                lp_vol: vec![
                    (String::from("MS "), 3),
                    (String::from("JPMC "), 1),
                    (String::from("CITI "), 5),
                ],
                volume: 9,
                price: 1.5556,
                side: String::from("Buy"),
            },
            FxAggBookEntry {
                lp_vol: vec![(String::from("UBS "), 1)],
                volume: 1,
                price: 1.5553,
                side: String::from("Buy"),
            },
            FxAggBookEntry {
                lp_vol: vec![
                    (String::from("UBS "), 3),
                    (String::from("CITI "), 1),
                    (String::from("BARX "), 1),
                    (String::from("BARX "), 5),
                ],
                volume: 10,
                price: 1.5554,
                side: String::from("Buy"),
            },
        ];

        sort_buy_book(&mut fx_buy_book);

        assert_eq!(
            fx_buy_book,
            vec![
                FxAggBookEntry {
                    lp_vol: vec![
                        (String::from("MS "), 3),
                        (String::from("JPMC "), 1),
                        (String::from("CITI "), 5),
                    ],
                    volume: 9,
                    price: 1.5556,
                    side: String::from("Buy"),
                },
                FxAggBookEntry {
                    lp_vol: vec![
                        (String::from("MS "), 1),
                        (String::from("UBS "), 5),
                        (String::from("CITI "), 3),
                        (String::from("BARX "), 3),
                    ],
                    volume: 12,
                    price: 1.5555,
                    side: String::from("Buy"),
                },
                FxAggBookEntry {
                    lp_vol: vec![
                        (String::from("UBS "), 3),
                        (String::from("CITI "), 1),
                        (String::from("BARX "), 1),
                        (String::from("BARX "), 5),
                    ],
                    volume: 10,
                    price: 1.5554,
                    side: String::from("Buy"),
                },
                FxAggBookEntry {
                    lp_vol: vec![(String::from("UBS "), 1)],
                    volume: 1,
                    price: 1.5553,
                    side: String::from("Buy"),
                },
            ]
        );
    }

    #[test]
    fn test_sort_by_price() {
        let mut fx_sell_book: Vec<FxAggBookEntry> = vec![
            FxAggBookEntry {
                lp_vol: vec![(String::from("MS "), 3), (String::from("JPMC "), 5)],
                volume: 8,
                price: 1.5565,
                side: String::from("Sell"),
            },
            FxAggBookEntry {
                lp_vol: vec![
                    (String::from("UBS "), 3),
                    (String::from("CITI "), 3),
                    (String::from("BARX "), 3),
                ],
                volume: 9,
                price: 1.5563,
                side: String::from("Sell"),
            },
            FxAggBookEntry {
                lp_vol: vec![(String::from("JPMC "), 1)],
                volume: 1,
                price: 1.5567,
                side: String::from("Sell"),
            },
            FxAggBookEntry {
                lp_vol: vec![
                    (String::from("MS "), 5),
                    (String::from("UBS "), 1),
                    (String::from("CITI "), 1),
                    (String::from("BARX "), 1),
                    (String::from("BARX "), 5),
                ],
                volume: 13,
                price: 1.5564,
                side: String::from("Sell"),
            },
            FxAggBookEntry {
                lp_vol: vec![(String::from("MS "), 1), (String::from("JPMC "), 3)],
                volume: 4,
                price: 1.5566,
                side: String::from("Sell"),
            },
        ];

        sort_sell_book(&mut fx_sell_book);
        assert_eq!(
            fx_sell_book,
            vec![
                FxAggBookEntry {
                    lp_vol: vec![
                        (String::from("UBS "), 3),
                        (String::from("CITI "), 3),
                        (String::from("BARX "), 3),
                    ],
                    volume: 9,
                    price: 1.5563,
                    side: String::from("Sell"),
                },
                FxAggBookEntry {
                    lp_vol: vec![
                        (String::from("MS "), 5),
                        (String::from("UBS "), 1),
                        (String::from("CITI "), 1),
                        (String::from("BARX "), 1),
                        (String::from("BARX "), 5),
                    ],
                    volume: 13,
                    price: 1.5564,
                    side: String::from("Sell"),
                },
                FxAggBookEntry {
                    lp_vol: vec![(String::from("MS "), 3), (String::from("JPMC "), 5)],
                    volume: 8,
                    price: 1.5565,
                    side: String::from("Sell"),
                },
                FxAggBookEntry {
                    lp_vol: vec![(String::from("MS "), 1), (String::from("JPMC "), 3)],
                    volume: 4,
                    price: 1.5566,
                    side: String::from("Sell"),
                },
                FxAggBookEntry {
                    lp_vol: vec![(String::from("JPMC "), 1)],
                    volume: 1,
                    price: 1.5567,
                    side: String::from("Sell"),
                },
            ]
        );
    }

    #[test]
    fn test_add_agg_book_entry() {
        let currency_pair = String::from("USD/EUR");
        let buy_book: Vec<FxAggBookEntry> = Vec::new();
        let sell_book: Vec<FxAggBookEntry> = Vec::new();
        let timestamp: u64 = 1753440851702449924;

        let mut fx_book = FxBook {
            currency_pair,
            buy_book,
            sell_book,
            timestamp,
        };

        add_agg_book_entry(&mut fx_book, "MS", 1, 1.5556, "Buy");

        assert_eq!(
            fx_book.buy_book,
            vec![FxAggBookEntry {
                lp_vol: vec![(String::from("MS "), 1),],
                volume: 1,
                price: 1.5556,
                side: String::from("Buy"),
            }]
        )
    }

    #[test]
    fn test_maintain_min_spread() {
        let mut fx_book = FxBook {
            currency_pair: String::from(" USD/EUR"),
            buy_book: vec![
                FxAggBookEntry {
                    lp_vol: vec![
                        (String::from("MS "), 1),
                        (String::from("UBS "), 5),
                        (String::from("CITI "), 3),
                        (String::from("BARX "), 3),
                    ],
                    volume: 12,
                    price: 1.5559,
                    side: String::from("Buy"),
                },
                FxAggBookEntry {
                    lp_vol: vec![
                        (String::from("MS "), 3),
                        (String::from("JPMC "), 1),
                        (String::from("CITI "), 5),
                    ],
                    volume: 9,
                    price: 1.5556,
                    side: String::from("Buy"),
                },
            ],
            sell_book: vec![FxAggBookEntry {
                lp_vol: vec![(String::from("MS "), 3), (String::from("JPMC "), 5)],
                volume: 8,
                price: 1.5564,
                side: String::from("Sell"),
            }],
            timestamp: 1753430617683973406,
        };

        maintain_min_spread(&mut fx_book);

        assert_eq!(
            fx_book.buy_book,
            vec![FxAggBookEntry {
                lp_vol: vec![
                    (String::from("MS "), 3),
                    (String::from("JPMC "), 1),
                    (String::from("CITI "), 5),
                ],
                volume: 9,
                price: 1.5556,
                side: String::from("Buy"),
            }]
        )
    }

    #[test]
    fn test_check_books_crossed() {
        let mut fx_book = FxBook {
            currency_pair: String::from(" USD/EUR"),
            buy_book: vec![
                FxAggBookEntry {
                    lp_vol: vec![
                        (String::from("MS "), 1),
                        (String::from("UBS "), 5),
                        (String::from("CITI "), 3),
                        (String::from("BARX "), 3),
                    ],
                    volume: 12,
                    price: 1.5559,
                    side: String::from("Buy"),
                },
                FxAggBookEntry {
                    lp_vol: vec![
                        (String::from("MS "), 3),
                        (String::from("JPMC "), 1),
                        (String::from("CITI "), 5),
                    ],
                    volume: 9,
                    price: 1.5556,
                    side: String::from("Buy"),
                },
            ],
            sell_book: vec![FxAggBookEntry {
                lp_vol: vec![(String::from("MS "), 3), (String::from("JPMC "), 5)],
                volume: 8,
                price: 1.5558,
                side: String::from("Sell"),
            }],
            timestamp: 1753430617683973406,
        };

        assert_eq!(check_books_crossed(&mut fx_book), Some((0, 1.5558)));
    }

    #[test]
    fn test_remove_entries_from_top() {
        let mut fx_buy_book: Vec<FxAggBookEntry> = vec![
            FxAggBookEntry {
                lp_vol: vec![
                    (String::from("MS "), 1),
                    (String::from("UBS "), 5),
                    (String::from("CITI "), 3),
                    (String::from("BARX "), 3),
                ],
                volume: 12,
                price: 1.5555,
                side: String::from("Buy"),
            },
            FxAggBookEntry {
                lp_vol: vec![
                    (String::from("MS "), 3),
                    (String::from("JPMC "), 1),
                    (String::from("CITI "), 5),
                ],
                volume: 9,
                price: 1.5556,
                side: String::from("Buy"),
            },
            FxAggBookEntry {
                lp_vol: vec![(String::from("UBS "), 1)],
                volume: 1,
                price: 1.5553,
                side: String::from("Buy"),
            },
            FxAggBookEntry {
                lp_vol: vec![
                    (String::from("UBS "), 3),
                    (String::from("CITI "), 1),
                    (String::from("BARX "), 1),
                    (String::from("BARX "), 5),
                ],
                volume: 10,
                price: 1.5554,
                side: String::from("Buy"),
            },
        ];
        remove_range_entries_from_top(&mut fx_buy_book, 1, "Buy");
        assert_eq!(
            fx_buy_book,
            vec![
                FxAggBookEntry {
                    lp_vol: vec![(String::from("UBS "), 1)],
                    volume: 1,
                    price: 1.5553,
                    side: String::from("Buy"),
                },
                FxAggBookEntry {
                    lp_vol: vec![
                        (String::from("UBS "), 3),
                        (String::from("CITI "), 1),
                        (String::from("BARX "), 1),
                        (String::from("BARX "), 5),
                    ],
                    volume: 10,
                    price: 1.5554,
                    side: String::from("Buy"),
                },
            ]
        )
    }

    #[test]
    fn test_check_expired_quotes() {
        let liquidity_provider = "JPMC ";
        let volume = 1;
        let mut fx_buy_book: Vec<FxAggBookEntry> = vec![
            FxAggBookEntry {
                lp_vol: vec![
                    (String::from("MS "), 1),
                    (String::from("UBS "), 5),
                    (String::from("CITI "), 3),
                    (String::from("BARX "), 3),
                ],
                volume: 12,
                price: 1.5555,
                side: String::from("Buy"),
            },
            FxAggBookEntry {
                lp_vol: vec![
                    (String::from("MS "), 3),
                    (String::from("JPMC "), 1),
                    (String::from("CITI "), 5),
                ],
                volume: 9,
                price: 1.5556,
                side: String::from("Buy"),
            },
            FxAggBookEntry {
                lp_vol: vec![(String::from("UBS "), 1)],
                volume: 1,
                price: 1.5553,
                side: String::from("Buy"),
            },
            FxAggBookEntry {
                lp_vol: vec![
                    (String::from("UBS "), 3),
                    (String::from("CITI "), 1),
                    (String::from("BARX "), 1),
                    (String::from("BARX "), 5),
                ],
                volume: 10,
                price: 1.5554,
                side: String::from("Buy"),
            },
        ];

        match check_expired_quotes(&mut fx_buy_book, liquidity_provider, volume) {
            Some(entry_to_remove) => {
                println!("test check expired quotes, index to remove is {entry_to_remove}")
            }
            None => (),
        };

        assert_eq!(
            fx_buy_book,
            vec![
                FxAggBookEntry {
                    lp_vol: vec![
                        (String::from("MS "), 1),
                        (String::from("UBS "), 5),
                        (String::from("CITI "), 3),
                        (String::from("BARX "), 3),
                    ],
                    volume: 12,
                    price: 1.5555,
                    side: String::from("Buy"),
                },
                FxAggBookEntry {
                    lp_vol: vec![(String::from("MS "), 3), (String::from("CITI "), 5),],
                    volume: 9,
                    price: 1.5556,
                    side: String::from("Buy"),
                },
                FxAggBookEntry {
                    lp_vol: vec![(String::from("UBS "), 1)],
                    volume: 1,
                    price: 1.5553,
                    side: String::from("Buy"),
                },
                FxAggBookEntry {
                    lp_vol: vec![
                        (String::from("UBS "), 3),
                        (String::from("CITI "), 1),
                        (String::from("BARX "), 1),
                        (String::from("BARX "), 5),
                    ],
                    volume: 10,
                    price: 1.5554,
                    side: String::from("Buy"),
                },
            ]
        )
    }
    #[test]
    fn test_find_buy_index_when_crossed() {
        let mut fx_book = FxBook {
            currency_pair: String::from(" USD/EUR"),
            buy_book: vec![
                FxAggBookEntry {
                    lp_vol: vec![
                        (String::from("MS "), 1),
                        (String::from("UBS "), 5),
                        (String::from("CITI "), 3),
                        (String::from("BARX "), 3),
                    ],
                    volume: 12,
                    price: 1.5566,
                    side: String::from("Buy"),
                },
                FxAggBookEntry {
                    lp_vol: vec![
                        (String::from("MS "), 3),
                        (String::from("JPMC "), 1),
                        (String::from("CITI "), 5),
                    ],
                    volume: 9,
                    price: 1.5565,
                    side: String::from("Buy"),
                },
                FxAggBookEntry {
                    lp_vol: vec![(String::from("UBS "), 1)],
                    volume: 1,
                    price: 1.5553,
                    side: String::from("Buy"),
                },
                FxAggBookEntry {
                    lp_vol: vec![
                        (String::from("UBS "), 3),
                        (String::from("CITI "), 1),
                        (String::from("BARX "), 1),
                        (String::from("BARX "), 5),
                    ],
                    volume: 10,
                    price: 1.5554,
                    side: String::from("Buy"),
                },
            ],
            sell_book: vec![
                FxAggBookEntry {
                    lp_vol: vec![(String::from("MS "), 3), (String::from("JPMC "), 5)],
                    volume: 8,
                    price: 1.5565,
                    side: String::from("Sell"),
                },
                FxAggBookEntry {
                    lp_vol: vec![
                        (String::from("UBS "), 3),
                        (String::from("CITI "), 3),
                        (String::from("BARX "), 3),
                    ],
                    volume: 9,
                    price: 1.5563,
                    side: String::from("Sell"),
                },
                FxAggBookEntry {
                    lp_vol: vec![(String::from("JPMC "), 1)],
                    volume: 1,
                    price: 1.5567,
                    side: String::from("Sell"),
                },
            ],
            timestamp: 1753430617683973406,
        };
        let fx_book_side = get_book_side(&mut fx_book, "Buy");
        assert_eq!(find_buy_index_when_crossed(fx_book_side, 1.5565), Some(1));
    }
}
