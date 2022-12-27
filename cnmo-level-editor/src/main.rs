use eframe::egui;
use cnmo_parse::lparse::level_data;

mod level_panel;
mod logger;
mod texture;
mod instanced_sprites;
mod common_gfx;
mod vertex;
mod tile_viewer;
mod camera;

#[derive(Debug, PartialEq, Eq)]
pub enum EditorMode {
    Level,
    Background,
    Tile,
}

struct LevelEditorApp {
    level_data: level_data::LevelData,
    mode: EditorMode,
    logs: logger::Logs,
}

impl LevelEditorApp {
    fn new(cc: &eframe::CreationContext, logs: logger::Logs) -> Self {
        common_gfx::GfxCommonResources::insert_resource(cc, "gfx.bmp");
        instanced_sprites::InstancedSpritesResources::insert_resource(cc);

        Self {
            level_data: level_data::LevelData::from_version(1).expect("Can't create empty level!"),
            mode: EditorMode::Level,
            logs,
        }
    }
}

impl eframe::App for LevelEditorApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::right("editor_options").resizable(true).max_width(500.0).show(ctx, |ui| {
            level_panel::show_metadata_panel(&mut self.mode, &mut self.level_data, ui);
        });
        egui::SidePanel::left("editor_properties").resizable(true).max_width(500.0).show(ctx, |ui| {
            level_panel::show_propeties_panel(&mut self.mode, &mut self.level_data, ui);
        });
        egui::TopBottomPanel::bottom("info_log").resizable(true).default_height(100.0).min_height(50.0).show(ctx, |ui| {
            logger::show_logs(&self.logs, ui);
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("central panel");
        });
    }
}

fn main() {
    let logger = Box::new(logger::Logger::new());
    let logs = std::sync::Arc::clone(&logger.logs);
    let _ = log::set_boxed_logger(logger)
        .map(|()| log::set_max_level(log::LevelFilter::Info));
    let native_options = eframe::NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        default_theme: eframe::Theme::Dark,
        ..Default::default()
    };
    eframe::run_native("CNM Online Level Editor", native_options, Box::new(|cc| Box::new(
        LevelEditorApp::new(cc, logs)
    )));
}
