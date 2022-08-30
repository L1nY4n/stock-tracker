use crate::back::message::Stock;
use crate::back::stork_api;
use std::thread;

use eframe::{
    egui::{
        self, menu, style::DebugOptions, Button, CentralPanel, CollapsingHeader, Context,
        CursorIcon, Frame, Label, Layout, RichText, Separator, SidePanel, Slider, Style, TextStyle,
        TopBottomPanel,
    },
    emath::Align,
    epaint::{Color32, Vec2},
    App, CreationContext,
};
use serde::{Deserialize, Serialize};

use super::back::{
    message::{ToBackend, ToFrontend},
    Back,
};
use crossbeam::channel::{Receiver, Sender};

#[derive(Default)]
pub struct StockTrackerApp {
    setting: Setting,
    stocks: Vec<Stock>,
    // Data transferring
    front_tx: Option<Sender<ToBackend>>,
    back_rx: Option<Receiver<ToFrontend>>,
}

#[derive(Default, Serialize, Deserialize)]
struct Setting {
    open: bool,
    show_color: bool,
    interval: u32,
    code: String,
}

// fn setup_custom_fonts(ctx: &egui::Context) {
//     let mut fonts = egui::FontDefinitions::default();
//     fonts.font_data.insert(
//         "my_font".to_owned(),
//         egui::FontData::from_static(include_bytes!(
//             "../../fonts/genShiGoThic/GenShinGothic-Medium.ttf"
//         )),
//     );

//     fonts
//         .families
//         .entry(egui::FontFamily::Monospace)
//         .or_default()
//         .insert(0, "my_font".to_owned());

//     fonts
//         .families
//         .entry(egui::FontFamily::Proportional)
//         .or_default()
//         .push("my_font".to_owned());

//     ctx.set_fonts(fonts);
// }

impl StockTrackerApp {
    pub fn new(cc: &CreationContext) -> Self {
        let mut new_app = Self::default();
        new_app.configure_style(&cc.egui_ctx);
        //setup_custom_fonts(&cc.egui_ctx);
        let (front_tx, front_rx) = crossbeam::channel::unbounded();
        let (back_tx, back_rx) = crossbeam::channel::unbounded();

        if let Some(storage) = cc.storage {
            if let Some(setting) = eframe::get_value(storage, eframe::APP_KEY) {
                new_app.setting = setting
            }
        }
        let codes = new_app.setting.code.clone();
        thread::spawn(|| Back::new(back_tx, front_rx, codes).init());
        new_app.setting.code = String::new();
        new_app.front_tx = Some(front_tx);
        new_app.back_rx = Some(back_rx);
        new_app
    }

    fn configure_style(&self, ctx: &Context) {
        let style = Style {
            debug: DebugOptions {
                debug_on_hover: false,
                show_expand_width: false,
                show_expand_height: false,
                show_resize: false,
            },
            ..Style::default()
        };
        ctx.set_style(style);
    }

    pub fn render_top_panel(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        // define a TopBottomPanel widget
        let f = Frame::none();
        TopBottomPanel::top("top_panel").frame(f).show(ctx, |ui| {
            ui.add_space(2.0);
            menu::bar(ui, |ui| {
                ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
                    ui.add_space(5.0);
                    let rocket_btn = ui
                        .add(Button::new(
                            RichText::new("💰")
                                .text_style(egui::TextStyle::Heading)
                                .color(Color32::YELLOW),
                        ))
                        .on_hover_cursor(CursorIcon::Move);
                    if rocket_btn.is_pointer_button_down_on() {
                        frame.drag_window()
                    };
                });
                // controls
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let close_btn = ui.add(Button::new(
                        RichText::new("❌")
                            .text_style(TextStyle::Body)
                            .color(Color32::RED),
                    ));
                    if close_btn.clicked() {
                        frame.close();
                    }

                    let refresh_btn = ui.add(Button::new(
                        RichText::new("🔄")
                            .text_style(TextStyle::Body)
                            .color(Color32::GREEN),
                    ));
                    if refresh_btn.clicked() {
                        self.stocks.clear();
                        if let Some(tx) = &self.front_tx {
                            tx.send(ToBackend::Refresh)
                                .expect("Failed sending refresh event.");
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
            ui.add_space(2.0);
            ui.add(Separator::default().spacing(0.0));
        });
    }

    fn render_stocks(&self, ui: &mut eframe::egui::Ui) {
        use egui_extras::{Size, TableBuilder};
        ui.add_space(2.0);
        TableBuilder::new(ui)
            .striped(true)
            .cell_layout(egui::Layout::left_to_right(Align::Center))
            .column(Size::initial(60.0))
            .column(Size::initial(40.0).at_least(20.0))
            .column(Size::remainder().at_least(30.0))
            .resizable(false)
            .body(|mut body| {
                for s in &self.stocks {
                    body.row(18.0, |mut row| {
                        row.col(|ui| {
                            ui.centered_and_justified(|ui| {
                                ui.add(Label::new(
                                    RichText::new(s.name.to_string())
                                        .text_style(egui::TextStyle::Body),
                                ));
                            });
                        });
                        row.col(|ui| {
                            ui.centered_and_justified(|ui| {
                                ui.add(Label::new(
                                    RichText::new(s.curr.to_string())
                                        .text_style(egui::TextStyle::Body),
                                ));
                            });
                        });
                        row.col(|ui| {
                            let color = if self.setting.show_color {
                                match s.percent {
                                    p if p < 0.0 => Color32::GREEN,
                                    n if n > 0.0 => Color32::RED,
                                    _ => Color32::WHITE,
                                }
                            } else {
                                Color32::WHITE
                            };
                            //   ui.centered_and_justified(|ui| {
                            ui.add(Label::new(
                                RichText::new(s.percent.to_string())
                                    .text_style(egui::TextStyle::Body)
                                    .color(color),
                            ));
                            //   });
                        });
                    })
                }
            });
    }

    fn setting_panel(&mut self, ctx: &eframe::egui::Context) {
        if self.setting.open {
            SidePanel::right("setting")
                .default_width(120.0)
                .frame(Frame::none().fill(Color32::from_black_alpha(255)))
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

                    self.setting_panel_contents(ui);
                });
        }
    }

    fn setting_panel_contents(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.spacing_mut().slider_width = 50.0;
            ui.label(RichText::new("🕘").color(Color32::GREEN));
            let interval_slider = ui.add(
                Slider::new(&mut self.setting.interval, 1..=30)
                    .suffix(" s")
                    .step_by(1.0),
            );
            if interval_slider.changed() {
                if let Some(tx) = &self.front_tx {
                    tx.send(ToBackend::SetInterval(self.setting.interval))
                        .expect("Failed sending  modify interval event .");
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
            ui.label(RichText::new("📓").color(Color32::GOLD));
            CollapsingHeader::new("stocks")
                .default_open(false)
                .show(ui, |ui| {
                    for s in &mut self.stocks {
                        ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                            ui.label(RichText::new(s.name.to_string()).color(Color32::LIGHT_BLUE));
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.add_space(3.0);
                                let close_btn = ui.add(Button::new(
                                    RichText::new("❌")
                                        .text_style(TextStyle::Body)
                                        .color(Color32::RED),
                                ));
                                if close_btn.clicked() {
                                    if let Some(tx) = &self.front_tx {
                                        tx.send(ToBackend::StockDel(s.code.clone()))
                                            .expect("Failed sending  SetCodel event .");
                                    };
                                }
                            });
                        });
                    }
                });
        });
        ui.add(Separator::default().spacing(0.0));
        ui.horizontal(|ui| {
            ui.label(RichText::new("➕").color(Color32::LIGHT_BLUE));
            let Self {
                setting: Setting { open, code, .. },
                ..
            } = self;
            let text_color = if stork_api::check_stock_code(&code) {
                Color32::GREEN
            } else {
                Color32::WHITE
            };
            let response = ui.add_sized(
                ui.available_size() - Vec2 { x: 2.0, y: 0.0 },
                egui::TextEdit::singleline(code).text_color(text_color),
            );

            if response.lost_focus() && ui.input().key_pressed(egui::Key::Enter) {
                if stork_api::check_stock_code(&code) {
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

impl App for StockTrackerApp {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        ctx.request_repaint();
        if let Some(rx) = &self.back_rx {
            match rx.try_recv() {
                Ok(message) => match message {
                    ToFrontend::DataList(d) => {
                        self.stocks = d;
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
                self.render_stocks(ui);
            });
        self.setting_panel(ctx);
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        let codes = self
            .stocks
            .iter()
            .map(|s| s.code.clone())
            .collect::<Vec<String>>()
            .join(",");
        self.setting.code = codes;
        eframe::set_value(storage, eframe::APP_KEY, &self.setting);
    }
}
