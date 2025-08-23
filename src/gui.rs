use crate::FxViewerApp;
use crate::aggregator;
use eframe::egui;
use egui::{Color32, Label, Layout, RichText};
use egui_extras::{TableBody, TableBuilder, TableRow};

pub fn render_top_panel(ctx: &egui::Context) {
    egui::TopBottomPanel::top("my_panel").show(ctx, |ui| {
        ctx.set_visuals(egui::Visuals::dark());
        ui.with_layout(Layout::left_to_right(eframe::emath::Align::Center), |ui| {
            ui.add_space(180.);
            ui.add(Label::new(
                RichText::new("Buy").text_style(egui::TextStyle::Heading),
            ));
            ui.add_space(160.);
            ui.add(Label::new(
                RichText::new("Sell").text_style(egui::TextStyle::Heading),
            ));
        });
    });
}

pub fn render_fx_book(fx_viewer_app: &mut FxViewerApp, ctx: &egui::Context) {
    let fx_book = fx_viewer_app.fx_book_mutex.lock().unwrap(); // panic if can't get lock
    //  println!("render_fx_book: fx_book: {:?}", fx_book);
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.with_layout(Layout::left_to_right(eframe::emath::Align::Center), |ui| {
            ui.with_layout(Layout::top_down(eframe::emath::Align::Center), |ui| {
                ui.push_id(1, |ui| {
                    TableBuilder::new(ui)
                        .id_salt(1)
                        .striped(true)
                        .columns(egui_extras::Column::auto().resizable(true), 3)
                        .cell_layout(egui::Layout::default().with_cross_align(egui::Align::Center))
                        .header(20.0, |header| {
                            render_buy_table_header(header);
                        })
                        .body(|body| {
                            render_buy_table_body(body, &fx_book.buy_book);
                        });
                });
            });
            // ui.add_space(10.);
            ui.with_layout(Layout::top_down(eframe::emath::Align::Center), |ui| {
                ui.push_id(2, |ui| {
                    TableBuilder::new(ui)
                        .id_salt(2)
                        .striped(true)
                        .columns(egui_extras::Column::auto().resizable(true), 3)
                        .cell_layout(egui::Layout::default().with_cross_align(egui::Align::Center))
                        .header(20.0, |header| {
                            render_sell_table_header(header);
                        })
                        .body(|body| {
                            render_sell_table_body(body, &fx_book.sell_book);
                        });
                });
            });
        });
    });
} // mutex lock released here

fn render_sell_table_header(mut header: TableRow<'_, '_>) {
    header.col(|ui| {
        ui.heading("Price");
    });
    header.col(|ui| {
        ui.heading("Volume (M)");
    });
    header.col(|ui| {
        ui.heading("");
    });
}

fn render_buy_table_header(mut header: TableRow<'_, '_>) {
    header.col(|ui| {
        ui.heading("");
    });
    header.col(|ui| {
        ui.heading("Volume (M)");
    });
    header.col(|ui| {
        ui.heading("Price");
    });
}

fn render_buy_table_body(mut body: TableBody<'_>, buy_book: &Vec<aggregator::FxAggBookEntry>) {
    for entry in buy_book {
        let lp_vol_vec = &entry.lp_vol;
        let len = lp_vol_vec.len() - 1;
        let mut lp_vol = String::from("(");
        body.row(30.0, |mut row| {
            row.col(|ui| {
                // ui.label(format!("{:?}", entry.lp_vol));

                let mut index = 0;
                for val in lp_vol_vec {
                    if index == 0 && len == 0 {
                        lp_vol = format!("{}{}: {})", lp_vol, val.0, val.1);
                        //  ui.label(lp_vol);
                    } else if index == 0 {
                        lp_vol = format!("{}{}: {},", lp_vol, val.0, val.1);
                        //  ui.label(lp_vol);
                    } else if index == len {
                        lp_vol = format!("{} {}: {})", lp_vol, val.0, val.1);
                        //  ui.label(lp_vol);
                    } else {
                        lp_vol = format!("{} {}: {},", lp_vol, val.0, val.1);
                        //  ui.label(lp_vol);
                    }
                    index += 1;
                }
                ui.label(lp_vol);
            });

            row.col(|ui| {
                ui.label(format!("{:?}", entry.volume));
            });
            row.col(|ui| {
                ui.label(RichText::new(format!("{:?}", entry.price)).color(Color32::GREEN));
            });
        });
    }
}

fn render_sell_table_body(mut body: TableBody<'_>, sell_book: &Vec<aggregator::FxAggBookEntry>) {
    for entry in sell_book {
        let lp_vol_vec = &entry.lp_vol;
        let len = lp_vol_vec.len() - 1;
        let mut lp_vol = String::from("(");
        body.row(30.0, |mut row| {
            row.col(|ui| {
                ui.label(RichText::new(format!("{:?}", entry.price)).color(Color32::GREEN));
            });

            row.col(|ui| {
                ui.label(format!("{:?}", entry.volume));
            });

            row.col(|ui| {
                // ui.label(format!("{:?}", entry.lp_vol));

                let mut index = 0;
                for val in lp_vol_vec {
                    if index == 0 && len == 0 {
                        lp_vol = format!("{}{}: {})", lp_vol, val.0, val.1);
                        //  ui.label(lp_vol);
                    } else if index == 0 {
                        lp_vol = format!("{}{}: {},", lp_vol, val.0, val.1);
                        //  ui.label(lp_vol);
                    } else if index == len {
                        lp_vol = format!("{} {}: {})", lp_vol, val.0, val.1);
                        //  ui.label(lp_vol);
                    } else {
                        lp_vol = format!("{} {}: {},", lp_vol, val.0, val.1);
                        //  ui.label(lp_vol);
                    }
                    index += 1;
                }
                ui.label(lp_vol);
            });
        });
    }
}
