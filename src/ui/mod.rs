use crate::back::stock::Stock;
use crate::back::stock::{self, KLineScale};
use std::{fmt::format, hash::Hash, thread, vec};

use eframe::{
    egui::{
        self, ahash::HashMap, menu, Align2, Button, CentralPanel, CollapsingHeader, Context,
        CursorIcon, Frame, Grid, Label, Layout, RichText, Separator, SidePanel, Slider, Stroke,
        Style, TextStyle, TopBottomPanel,
    },
    emath::Align,
    epaint::{Color32, Vec2},
    App, CreationContext,
};
use egui_plot::{
    AxisHints, Bar, BarChart, BoxElem, BoxPlot, BoxSpread, CoordinatesFormatter, Corner, HLine,
    Plot, PlotPoint, Text, VLine,
};
use serde::{Deserialize, Serialize};

use super::back::{
    message::{ToBackend, ToFrontend},
    Back,
};
use crossbeam::channel::{Receiver, Sender};

#[derive(Default)]
pub struct StockTrackerApp {
    time: String,
    setting: Setting,
    stocks: HashMap<String, Stock>,
    // Data transferring
    front_tx: Option<Sender<ToBackend>>,
    back_rx: Option<Receiver<ToFrontend>>,
}

#[derive(Default, Serialize, Deserialize)]
struct Setting {
    open: bool,
    show_name: bool,
    show_color: bool,
    hide_name: bool,
    interval: u32,
    stocks: String,
    adding_code: String,
}

impl StockTrackerApp {
    pub fn new(cc: &CreationContext) -> Self {
        let mut app = Self::default();
        app.configure_style(&cc.egui_ctx);
        load_font(&cc.egui_ctx);
        cc.egui_ctx.set_theme(egui::Theme::Dark);
        let (front_tx, front_rx) = crossbeam::channel::unbounded();
        let (back_tx, back_rx) = crossbeam::channel::unbounded();

        if let Some(storage) = cc.storage {
            if let Some(setting) = eframe::get_value(storage, eframe::APP_KEY) {
                app.setting = setting
            }
        }
        let codes = app.setting.stocks.clone();
        thread::spawn(|| Back::new(back_tx, front_rx, codes).run());
        app.front_tx = Some(front_tx);
        app.back_rx = Some(back_rx);
        app
    }

    fn configure_style(&self, ctx: &Context) {
        let style = Style { ..Style::default() };
        ctx.set_style(style);
    }

    fn render_top_panel(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        // define a TopBottomPanel widget
        let f = Frame::none();
        TopBottomPanel::top("top_panel").frame(f).show(ctx, |ui| {
            ui.add_space(2.0);
            menu::bar(ui, |ui| {
                ui.with_layout(Layout::left_to_right(Align::BOTTOM), |ui| {
                    ui.add_space(5.0);
                    let rocket_btn = ui
                        .add(Button::new(
                            RichText::new("🚀")
                                .text_style(egui::TextStyle::Heading)
                                .color(Color32::YELLOW),
                        ))
                        .on_hover_cursor(CursorIcon::Move);
                    if rocket_btn.is_pointer_button_down_on() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                    };

                    ui.add(Label::new(
                        RichText::new(self.time.clone())
                        .text_style(egui::TextStyle::Small)
                 
                    )
                    );
                });
                // controls
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let close_btn = ui.add(Button::new(
                        RichText::new("❌")
                            .text_style(TextStyle::Body)
                            .color(Color32::RED),
                    ));
                    if close_btn.clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }

                    let refresh_btn = ui.add(Button::new(
                        RichText::new("🔄")
                            .text_style(TextStyle::Body)
                            .color(Color32::GREEN),
                    ));
                    if refresh_btn.clicked() {
                        self.stocks.clear();
                        if let Some(tx) = &self.front_tx {
                            let _ = tx.send(ToBackend::Refresh);
                        }
                    }

                    // config button
                    let config_btn = ui.add(Button::new(
                        RichText::new("🛠")
                            .text_style(egui::TextStyle::Body)
                            .color(Color32::LIGHT_BLUE),
                    ));

                    if config_btn.clicked() {
                        self.setting.open = !self.setting.open
                    }
                });
            });

            ui.add(Separator::default().spacing(0.0));
        });
    }

    fn render_stocks(&mut self, ctx: &Context, ui: &mut eframe::egui::Ui) {
        ui.add_space(2.0);
        Grid::new("stock_grid")
            .max_col_width(56.0)
            .striped(true)
            .show(ui, |ui| {
                for stock in self.stocks.values_mut().into_iter() {
                    ui.centered_and_justified(|ui| {
                        if self.setting.show_name {
                            ui.add(
                                Label::new(
                                    RichText::new(stock.name.to_string())
                                        .text_style(egui::TextStyle::Body),
                                )
                                .wrap_mode(egui::TextWrapMode::Truncate),
                            )
                        } else {
                            ui.add(Label::new(
                                RichText::new("   ").text_style(egui::TextStyle::Body),
                            ))
                        }
                    });
                    ui.centered_and_justified(|ui| {
                        ui.add(Label::new(
                            RichText::new(stock.data_new().to_string())
                                .text_style(egui::TextStyle::Body),
                        ));
                    });
                    let color = if self.setting.show_color {
                        match stock.data_rise_per() {
                            p if p < 0.0 => Color32::GREEN,
                            n if n > 0.0 => Color32::RED,
                            _ => Color32::WHITE,
                        }
                    } else {
                        Color32::WHITE
                    };
                    ui.centered_and_justified(|ui| {
                        ui.add(Label::new(
                            RichText::new(stock.data_rise_per().to_string())
                                .text_style(egui::TextStyle::Body)
                                .color(color),
                        ))
                    });

                    ui.centered_and_justified(|ui| {
                        let boxs = stock
                            .klines
                            .iter()
                            .enumerate()
                            .map(|(i, x)| {
                                let fill_color = if x.close < x.open {
                                    Color32::GREEN
                                } else {
                                    Color32::RED
                                };

                                BoxElem::new(
                                    i as f64,
                                    BoxSpread::new(
                                        x.low,
                                        x.open,
                                        (x.open + x.close) / 2.0,
                                        x.close,
                                        x.high,
                                    ),
                                )
                                .stroke(Stroke::new(0.2, fill_color))
                                .fill(fill_color.linear_multiply(0.1))
                                .box_width(0.8)
                            })
                            .collect();
                        let box1 = BoxPlot::new(boxs);

                        let plot = Plot::new(format!("{}_kline", stock.code))
                            .allow_zoom(false)
                            .allow_drag(false)
                            .allow_scroll(false)
                            .show_grid([false, false])
                            .show_axes([false, false])
                            .sharp_grid_lines(false)
                            // .show_background(false)
                            .width(50.0)
                            .height(16.0)
                            .show(ui, |plot_ui| {
                                plot_ui.box_plot(box1);
                            })
                            .response;

                        if plot.clicked() {
                            if let Some(tx) = &self.front_tx {
                                let _ = tx.send(ToBackend::StockKLine(
                                    stock.code.to_string(),
                                    stock.kline_scale.clone(),
                                ));
                            }
                            stock.show_klines_viewport = true;
                        }

                        if stock.show_klines_viewport {
                            ctx.show_viewport_immediate(
                                egui::ViewportId::from_hash_of(format!("{}_kline_v", stock.code)),
                                egui::ViewportBuilder::default()
                                    .with_title(format!("{}", stock.code)),
                                |ctx, _class| {
                                    egui::CentralPanel::default().show(ctx, |ui| {
                                        ui.vertical(|ui| {
                                            ui.horizontal_wrapped(|ui| {
                                                if ui
                                                    .selectable_value(
                                                        &mut stock.kline_scale,
                                                        KLineScale::Munute5,
                                                        "5",
                                                    )
                                                    .clicked()
                                                {
                                                    if let Some(tx) = &self.front_tx {
                                                        let _ = tx.send(ToBackend::StockKLine(
                                                            stock.code.to_string(),
                                                            KLineScale::Munute5,
                                                        ));
                                                    }
                                                }
                                                if ui
                                                    .selectable_value(
                                                        &mut stock.kline_scale,
                                                        KLineScale::Munute15,
                                                        "15",
                                                    )
                                                    .clicked()
                                                {
                                                    if let Some(tx) = &self.front_tx {
                                                        let _ = tx.send(ToBackend::StockKLine(
                                                            stock.code.to_string(),
                                                            KLineScale::Munute15,
                                                        ));
                                                    }
                                                }
                                                if ui
                                                    .selectable_value(
                                                        &mut stock.kline_scale,
                                                        KLineScale::Munute30,
                                                        "30",
                                                    )
                                                    .clicked()
                                                {
                                                    if let Some(tx) = &self.front_tx {
                                                        let _ = tx.send(ToBackend::StockKLine(
                                                            stock.code.to_string(),
                                                            KLineScale::Munute30,
                                                        ));
                                                    }
                                                }
                                                if ui
                                                    .selectable_value(
                                                        &mut stock.kline_scale,
                                                        KLineScale::Day,
                                                        "day",
                                                    )
                                                    .clicked()
                                                {
                                                    if let Some(tx) = &self.front_tx {
                                                        let _ = tx.send(ToBackend::StockKLine(
                                                            stock.code.to_string(),
                                                            KLineScale::Day,
                                                        ));
                                                    }
                                                }
                                                if ui
                                                    .selectable_value(
                                                        &mut stock.kline_scale,
                                                        KLineScale::Week,
                                                        "week",
                                                    )
                                                    .clicked()
                                                {
                                                    if let Some(tx) = &self.front_tx {
                                                        let _ = tx.send(ToBackend::StockKLine(
                                                            stock.code.to_string(),
                                                            KLineScale::Week,
                                                        ));
                                                    }
                                                }
                                            });

                                            let mut x_axes = vec![];
                                            let boxs = stock
                                                .klines
                                                .iter()
                                                .enumerate()
                                                .map(|(i, x)| {
                                                    let x_hints = AxisHints::new_x().label("Time");
                                                    x_axes.push(x_hints);

                                                    let fill_color = if x.close < x.open {
                                                        Color32::GREEN
                                                    } else {
                                                        Color32::RED
                                                    };
                                                    BoxElem::new(
                                                        i as f64,
                                                        BoxSpread::new(
                                                            x.low,
                                                            x.open,
                                                            (x.open + x.close) / 2.0,
                                                            x.close,
                                                            x.high,
                                                        ),
                                                    )
                                                    .stroke(Stroke::new(0.2, fill_color))
                                                    .fill(fill_color.linear_multiply(0.05))
                                                    .box_width(0.8)
                                                })
                                                .collect();
                                            let box1 = BoxPlot::new(boxs);

                                            Plot::new(format!("{}_kline", stock.code))
                                                .show_background(false)
                                                .show_grid(true)
                                                .allow_drag([true, false])
                                                .custom_x_axes(vec![
                                                    AxisHints::new_x().label("Time (s)")
                                                ])
                                                .show(ui, |plot_ui| {
                                                    plot_ui.box_plot(box1);
                                                })
                                                .response;
                                        });
                                    });

                                    if ctx.input(|i| i.viewport().close_requested()) {
                                        // Tell parent viewport that we should not show next frame:
                                        stock.show_klines_viewport = false;
                                    }
                                },
                            );
                        }
                    });

                    ui.centered_and_justified(|ui| {
                        let bids_bars = stock
                            .data_bids()
                            .iter()
                            .map(|(v, p)| {
                                Bar::new((p - stock.data_new()).into(), *v as f64).width(0.001)
                            })
                            .collect();

                        let bid_chart = BarChart::new(bids_bars)
                            .allow_hover(false)
                            .color(Color32::LIGHT_GREEN);

                        let asks_bar = stock
                            .data_asks()
                            .iter()
                            .map(|(v, p)| {
                                Bar::new((p - stock.data_new()).into(), *v as f64).width(0.001)
                            })
                            .collect();
                        let ask_chart = BarChart::new(asks_bar)
                            .allow_hover(false)
                            .color(Color32::YELLOW);

                        let plot = Plot::new(stock.code.to_string())
                            .allow_zoom(false)
                            .allow_drag(false)
                            .allow_scroll(false)
                            .show_grid([false, false])
                            .show_axes([false, false])
                            .sharp_grid_lines(false)
                            .width(50.0)
                            .height(16.0)
                            .center_x_axis(true)
                            .show(ui, |plot_ui| {
                                plot_ui.bar_chart(bid_chart);
                                plot_ui.bar_chart(ask_chart)
                            })
                            .response;

                        plot.on_hover_ui(|ui| {
                            ui.vertical(|ui| {
                                ui.group(|ui| {
                                    ui.set_max_size(Vec2::new(200.0, 120.0));

                                    let mut bids_text = vec![];
                                    let bids = stock
                                        .data_bids()
                                        .iter()
                                        .map(|(v, p)| {
                                            bids_text.push(
                                                Text::new(
                                                    PlotPoint::new(-10.0, p - stock.data_new()),
                                                    format!(
                                                        "{:.2}    {}  -  {}  ",
                                                        (*v as f32) * (*p) * 0.01,
                                                        v,
                                                        p
                                                    ),
                                                )
                                                .anchor(Align2::RIGHT_CENTER),
                                            );
                                            Bar::new((p - stock.data_new()).into(), *v as f64)
                                                .width(0.001)
                                        })
                                        .collect();

                                    let mut asks_text = vec![];
                                    let asks = stock
                                        .data_asks()
                                        .iter()
                                        .map(|(v, p)| {
                                            asks_text.push(
                                                Text::new(
                                                    PlotPoint::new(-10.0, p - stock.data_new()),
                                                    format!(
                                                        "{:.2}    {}  -  {}  ",
                                                        (*v as f32) * (*p) * 0.01,
                                                        v,
                                                        p
                                                    ),
                                                )
                                                .anchor(Align2::RIGHT_CENTER),
                                            );
                                            Bar::new((p - stock.data_new()).into(), *v as f64)
                                                .width(0.001)
                                        })
                                        .collect();

                                    let bid_chart =
                                        BarChart::new(bids).color(Color32::GREEN).horizontal();
                                    let ask_chart =
                                        BarChart::new(asks).color(Color32::YELLOW).horizontal();

                                    Plot::new(stock.code.to_string())
                                        .show_grid(false)
                                        .show_axes([false, false])
                                        .sharp_grid_lines(false)
                                        .show_background(false)
                                        .show_x(false)
                                        .show_y(true)
                                        .center_x_axis(true)
                                        .show(ui, |plot_ui| {
                                            plot_ui.bar_chart(bid_chart);
                                            plot_ui.bar_chart(ask_chart);
                                            plot_ui.hline(
                                                HLine::new(0.0)
                                                    .color(Color32::GRAY.linear_multiply(0.05)),
                                            );
                                            bids_text.iter().for_each(|t| {
                                                plot_ui.text(t.clone());
                                            });
                                            asks_text.iter().for_each(|t| {
                                                plot_ui.text(t.clone());
                                            })
                                        })
                                        .response
                                });
                            });
                        });
                    });

                    ui.end_row();
                }
            });
    }

    fn setting_panel(&mut self, ctx: &eframe::egui::Context) {
        if self.setting.open {
            SidePanel::right("setting")
                .frame(
                    Frame::none()
                        .fill(Color32::from_black_alpha(172))
                        .inner_margin(4.0),
                )
                .show(ctx, |ui| {
                    menu::bar(ui, |ui| {
                        ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                            ui.label(RichText::new("⚙ setting").color(Color32::LIGHT_BLUE));
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                let close_btn = ui.add(Button::new(
                                    RichText::new("\u{2bab}")
                                        .text_style(TextStyle::Body)
                                        .color(Color32::GRAY),
                                ));
                                if close_btn.clicked() {
                                    self.setting.open = false
                                }
                            });
                        });
                    });

                    self.setting_panel_contents(ctx, ui);
                });
        }
    }

    fn setting_panel_contents(&mut self, ctx: &eframe::egui::Context, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.spacing_mut().slider_width = 50.0;
            ui.label(RichText::new("🕘").color(Color32::GREEN));
            let interval_slider = ui.add(
                Slider::new(&mut self.setting.interval, 200..=1000)
                    .suffix(" ms")
                    .step_by(100.0),
            );
            if interval_slider.changed() {
                if let Some(tx) = &self.front_tx {
                    let _ = tx.send(ToBackend::SetInterval(self.setting.interval));
                }
            }
        });
        ui.add(Separator::default().spacing(0.0));

        ui.horizontal(|ui| {
            ui.label(RichText::new("🎨").color(Color32::GOLD));
            ui.checkbox(&mut self.setting.show_color, "color");
        });
        ui.add(Separator::default().spacing(0.0));

        ui.horizontal(|ui| {
            ui.label(RichText::new("👓").color(Color32::KHAKI));
            ui.checkbox(&mut self.setting.show_name, "name");
        });
        ui.add(Separator::default().spacing(0.0));

        ui.horizontal(|ui| {
            ui.label(RichText::new("📓").color(Color32::LIGHT_BLUE));
            CollapsingHeader::new("stocks")
                .default_open(false)
                .show(ui, |ui| {
                    for s in self.stocks.clone().values().into_iter() {
                        ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                            ui.label(
                                RichText::new(format!("{}({})", s.name, s.code))
                                    .color(Color32::LIGHT_BLUE),
                            );
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.add_space(3.0);
                                let close_btn = ui.add(Button::new(
                                    RichText::new("❌")
                                        .text_style(TextStyle::Body)
                                        .color(Color32::RED),
                                ));
                                if close_btn.clicked() {
                                    self.stocks.remove(&s.code);
                                    if let Some(tx) = &self.front_tx {
                                        let _ = tx.send(ToBackend::StockDel(s.code.clone()));
                                    };
                                }
                            });
                        });
                    }
                });
        });
        ui.add(Separator::default().spacing(0.0));
        ui.horizontal(|ui| {
            ui.label(RichText::new("➕").color(Color32::LIGHT_GRAY));
            let Self {
                setting:
                    Setting {
                        open,
                        adding_code: code,
                        ..
                    },
                ..
            } = self;
            let text_color = if stock::check_stock_code(&code) {
                Color32::GREEN
            } else {
                Color32::WHITE
            };
            let response = ui.add_sized(
                ui.available_size() - Vec2 { x: 2.0, y: 0.0 },
                egui::TextEdit::singleline(code).text_color(text_color),
            );

            if response.lost_focus() || ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                if stock::check_stock_code(&code) {
                    if let Some(tx) = &self.front_tx {
                        tx.send(ToBackend::StockAdd(code.to_string()))
                            .expect("Failed sending  add stock event .");
                        *code = String::new();
                        *open = false;
                    };
                };
            };
        });
        ui.add(Separator::default().spacing(0.0));
    }
}

impl StockTrackerApp {
    fn update_time(&mut self) {
        self.time = format!("{}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"));
    }
}

impl App for StockTrackerApp {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        ctx.request_repaint();
        if let Some(rx) = &self.back_rx {
            match rx.try_recv() {
                Ok(message) => match message {
                    ToFrontend::DataList(list) => {
                        self.update_time();
                        list.iter().for_each(|(code, name, base_data)| {
                            if let Some(s) = self.stocks.get_mut(code) {
                                s.set_data(base_data.clone());
                            } else {
                                let mut s = Stock::new(&code, &name);
                                s.set_data(base_data.clone());
                                self.stocks.insert(code.to_string(), s);
                            }
                        });
                    }
                    ToFrontend::Data(code, name, base_data) => {
                        if let Some(s) = self.stocks.get_mut(&code) {
                            s.set_data(base_data);
                        } else {
                            let mut s = Stock::new(&code, &name);
                            s.set_data(base_data);
                            self.stocks.insert(code, s);
                        }
                    }
                    ToFrontend::Kline(code, klines) => {
                        if let Some(s) = self.stocks.get_mut(&code) {
                            s.set_klines(klines)
                        }
                    }
                },
                Err(err) => {
                    let _ = err;
                }
            }
        }
        self.render_top_panel(ctx, frame);

        CentralPanel::default()
            .frame(Frame::none())
            .show(ctx, |ui| {
                self.render_stocks(ctx, ui);
            });
        self.setting_panel(ctx);
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        let codes = self
            .stocks
            .values()
            .into_iter()
            .map(|s| s.code.clone())
            .collect::<Vec<String>>()
            .join(",");
        self.setting.stocks = codes;
        eframe::set_value(storage, eframe::APP_KEY, &self.setting);
    }
}

fn load_font(ctx: &egui::Context) {
    let mut fonts = eframe::egui::FontDefinitions::default();

    fonts.font_data.insert(
        "AlibabaPuHuiTi-3-55-Regular".to_owned(),
        eframe::egui::FontData::from_static(include_bytes!(
            "../../resources/AlibabaPuHuiTi-3-55-Regular.ttf"
        )),
    ); // .ttf and .otf supported

    fonts
        .families
        .get_mut(&eframe::egui::FontFamily::Proportional)
        .unwrap()
        .insert(0, "AlibabaPuHuiTi-3-55-Regular".to_owned());
    ctx.set_fonts(fonts);
}
