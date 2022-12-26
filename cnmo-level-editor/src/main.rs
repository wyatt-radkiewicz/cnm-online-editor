use eframe::egui;
use cnmo_parse::lparse::level_data;

mod level_panel;

#[derive(Debug, PartialEq, Eq)]
pub enum EditorMode {
    Level,
    Background,
    Tile,
}

struct LevelEditorApp {
    level_data: level_data::LevelData,
    mode: EditorMode,
}

impl LevelEditorApp {
    fn new() -> Self {
        Self {
            level_data: level_data::LevelData::from_version(1).expect("Can't create empty level!"),
            mode: EditorMode::Level,
        }
    }
}

impl eframe::App for LevelEditorApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::right("editor_options").resizable(true).max_width(500.0).show(ctx, |ui| {
            level_panel::show_metadata_panel(&mut self.mode, &mut self.level_data, ui);
        });
        egui::SidePanel::left("editor_properties").resizable(true).max_width(500.0).show(ctx, |ui| {
            level_panel::show_properties_panel(&mut self.mode, &mut self.level_data, ui);
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("central panel");
        });
    }
}

fn main() {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native("CNM Online Level Editor", native_options, Box::new(|_cc| Box::new(
        LevelEditorApp::new()
    )));
}
