#[cfg(test)]
mod tests {

    use crate::aggregator::FxAggBookEntry;
    use crate::aggregator::{self, FxBook};

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

        aggregator::sort_buy_book(&mut fx_buy_book);

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

        aggregator::sort_sell_book(&mut fx_sell_book);
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

        aggregator::add_agg_book_entry(&mut fx_book, "MS", 1, 1.5556, "Buy");

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

        aggregator::maintain_min_spread(&mut fx_book);

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

        assert_eq!(
            aggregator::check_books_crossed(&mut fx_book),
            Some((0, 1.5558))
        );
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
        aggregator::remove_range_entries_from_top(&mut fx_buy_book, 1, "Buy");
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

        match aggregator::check_expired_quotes(&mut fx_buy_book, liquidity_provider, volume) {
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
        let fx_book_side = aggregator::get_book_side(&mut fx_book, "Buy");
        assert_eq!(
            aggregator::find_buy_index_when_crossed(fx_book_side, 1.5565),
            Some(1)
        );
    }
}
