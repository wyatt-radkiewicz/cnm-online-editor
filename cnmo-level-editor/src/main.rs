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
mod editor_data;
mod world_panel;
mod tile_panel;
mod bgpanel;

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
    editor_data: editor_data::EditorData,
    properties_panel: level_panel::PropertiesPanel,
    world_panel: world_panel::WorldPanel,
    tile_panel: tile_panel::TilePanel,
    bg_panel: bgpanel::BgPanel,
}

impl LevelEditorApp {
    fn new(cc: &eframe::CreationContext, logs: logger::Logs) -> Self {
        let (palette, dimensions, opaques) = common_gfx::GfxCommonResources::insert_resource(cc, "gfx.bmp");
        instanced_sprites::InstancedSpritesResources::<tile_viewer::TileViewerSpriteInstances>::insert_resource(cc);
        instanced_sprites::InstancedSpritesResources::<tile_viewer::TileViewerDraggingSpriteInstances>::insert_resource(cc);
        instanced_sprites::InstancedSpritesResources::<world_panel::WorldPanelSpriteInstances>::insert_resource(cc);
        instanced_sprites::InstancedSpritesResources::<tile_panel::TilePanelSpriteInstances>::insert_resource(cc);
        instanced_sprites::InstancedSpritesResources::<tile_panel::TilePanelCollisionDataSpriteInstances>::insert_resource(cc);
        instanced_sprites::InstancedSpritesResources::<tile_panel::TilePanelPreviewSpriteInstances>::insert_resource(cc);
        instanced_sprites::InstancedSpritesResources::<bgpanel::BgPanelSpriteInstances>::insert_resource(cc);

        Self {
            level_data: level_data::LevelData::from_version(1).expect("Can't create empty level!"),
            mode: EditorMode::Level,
            logs,
            editor_data: editor_data::EditorData::new(palette, dimensions, opaques),
            properties_panel: level_panel::PropertiesPanel::new(),
            world_panel: world_panel::WorldPanel::new(),
            tile_panel: tile_panel::TilePanel::new(),
            bg_panel: bgpanel::BgPanel::new(),
        }
    }
}

impl eframe::App for LevelEditorApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();
        self.editor_data.update_delta_time();
        egui::SidePanel::right("editor_options").resizable(true).max_width(500.0).show(ctx, |ui| {
            level_panel::show_metadata_panel(&mut self.world_panel, &mut self.editor_data, &mut self.mode, &mut self.level_data, ui, &mut self.bg_panel);
        });
        egui::SidePanel::left("editor_properties").resizable(true).max_width(500.0).show(ctx, |ui| {
            self.properties_panel.show_propeties_panel(&mut self.editor_data, &mut self.mode, &mut self.level_data, ui, &mut self.world_panel);
        });
        egui::TopBottomPanel::bottom("info_log").resizable(true).default_height(100.0).min_height(50.0).show(ctx, |ui| {
            logger::show_logs(&self.logs, ui);
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.mode {
                EditorMode::Background => {
                    if self.editor_data.selecting_background_color || self.editor_data.selecting_background_image {
                        self.bg_panel.update(ui, &mut self.level_data, &mut self.editor_data);
                    } else {
                        self.world_panel.update(ui, &mut self.level_data, &mut self.editor_data);
                    }
                },
                EditorMode::Level => {
                    self.world_panel.update(ui, &mut self.level_data, &mut self.editor_data);
                },
                EditorMode::Tile => {
                    self.tile_panel.update(ui, &mut self.level_data, &mut self.editor_data);
                },
            }
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
        min_window_size: Some(egui::vec2(800.0, 600.0)),
        ..Default::default()
    };
    eframe::run_native("CNM Online Level Editor", native_options, Box::new(|cc| Box::new(
        LevelEditorApp::new(cc, logs)
    )));
}
