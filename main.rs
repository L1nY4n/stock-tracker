#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // windows_subsystem 告诉编译器，程序运行时隐藏命令行窗口。
use eframe::egui;
use native_dialog::{FileDialog, MessageDialog, MessageType};
fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(320.0, 340.0)), //初始化窗体size
        ..Default::default()
    };
    eframe::run_native(
        "Hello word", //应用程序名称
        options,
        Box::new(|_cc| Box::<MyApp>::new(MyApp::new(_cc))), //第三个参数为程序构建器(eframe::AppCreator类型)负责创建应用程序上下文(egui::Context)。_cc为&CreationContextl类型，_cc.egui_ctx字段即为Context。
                                                            //之所以强调Context的创建过程，是因为显示中文字体需要配置Context。
    )
}

struct MyApp {
    name: String,
    age: u32,
}
impl Default for MyApp {
    fn default() -> Self {
        Self {
            name: "Arthur".to_owned(),
            age: 42,
        }
    }
}
impl MyApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        load_font(&cc.egui_ctx); //egui默认字体无法显示中文，需要加载中文字体。配置字体应该在构造函数中。网上部分教程将字体配置写入了update函数，update函数每一帧都会运行一次，每秒60次，因此在update函数中加载字体是错误且低效的。
        Self::default()
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Egui 0.23 Application");
            ui.horizontal(|ui| {
                let name_label = ui.label("Your name: ");
                ui.text_edit_multiline(&mut self.name)
                    .labelled_by(name_label.id);
            });
            ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));
            if ui.button("Open file").clicked() {
                let path = FileDialog::new()
                    .set_location("~/Desktop")
                    .add_filter("PNG Image", &["png"])
                    .add_filter("JPEG Image", &["jpg", "jpeg"])
                    .show_open_single_file()
                    .unwrap();
                    if let Some(path) =path{
                        self.name=path.as_path().display().to_string();
                    }
            }
            ui.label(format!("Hello '{}', age {}", self.name, self.age));
        });
    }
}

// 为了支持中文，我们加载阿里巴巴普惠体字体：下载自https://fonts.alibabagroup.com/#/home
//将字体文件放置在src目录同级别的resources目录下
pub fn load_font(ctx: &egui::Context) {
    let mut fonts = eframe::egui::FontDefinitions::default();
    fonts.font_data.insert(
        "AlibabaPuHuiTi-3-55-Regular".to_owned(),
        eframe::egui::FontData::from_static(include_bytes!(
            "../resources/AlibabaPuHuiTi-3-55-Regular.ttf"
        )),
    ); // .ttf and .otf supported

    fonts
        .families
        .get_mut(&eframe::egui::FontFamily::Proportional)
        .unwrap()
        .insert(0, "AlibabaPuHuiTi-3-55-Regular".to_owned());
    ctx.set_fonts(fonts);
}
