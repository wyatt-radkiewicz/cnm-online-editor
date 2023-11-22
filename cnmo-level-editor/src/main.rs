#![windows_subsystem = "windows"]


use notify::RecursiveMode;
use std::sync::mpsc::Receiver;

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
mod game_config_panel;

#[derive(Debug, PartialEq, Eq)]
pub enum EditorMode {
    Level,
    Background,
    Tile,
    GameConfig,
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
    game_config_panel: game_config_panel::GameConfigPanel,
    render_state: eframe::egui_wgpu::RenderState,
    file_receiver: Receiver<notify_debouncer_mini::DebounceEventResult>,
    _debouncer: notify_debouncer_mini::Debouncer<notify::RecommendedWatcher>,
    watch_timer: std::time::Duration,
}

impl LevelEditorApp {
    fn new(cc: &eframe::CreationContext, logs: logger::Logs) -> Self {
        let render_state = cc.wgpu_render_state.as_ref().expect("Need a WGPU rendering context for editor....").clone();
        let (palette, dimensions, opaques) = common_gfx::GfxCommonResources::insert_resource(&render_state, "gfx.bmp");
        instanced_sprites::InstancedSpritesResources::<tile_viewer::TileViewerSpriteInstances>::insert_resource(cc);
        instanced_sprites::InstancedSpritesResources::<tile_viewer::TileViewerDraggingSpriteInstances>::insert_resource(cc);
        instanced_sprites::InstancedSpritesResources::<world_panel::WorldPanelSpriteInstances>::insert_resource(cc);
        instanced_sprites::InstancedSpritesResources::<tile_panel::TilePanelSpriteInstances>::insert_resource(cc);
        instanced_sprites::InstancedSpritesResources::<tile_panel::TilePanelCollisionDataSpriteInstances>::insert_resource(cc);
        instanced_sprites::InstancedSpritesResources::<tile_panel::TilePanelPreviewSpriteInstances>::insert_resource(cc);
        instanced_sprites::InstancedSpritesResources::<bgpanel::BgPanelSpriteInstances>::insert_resource(cc);
        instanced_sprites::InstancedSpritesResources::<game_config_panel::GfxPreviewResource>::insert_resource(cc);
        instanced_sprites::InstancedSpritesResources::<level_panel::LevelIconPreviewSpriteInstances>::insert_resource(cc);

        let _config = notify::Config::default();
        let (tx, file_receiver) = std::sync::mpsc::channel();
        let mut debouncer =
            notify_debouncer_mini::new_debouncer(std::time::Duration::from_secs(2), None, tx)
            .expect("Need hot-realoading file watching for editor to boot");
        debouncer.watcher().watch(&std::path::Path::new("./"), RecursiveMode::Recursive).expect("Can't open working directory for hot-reloading");

        Self {
            level_data: level_data::LevelData::from_version(1).expect("Can't create empty level!"),
            mode: EditorMode::Level,
            logs,
            editor_data: editor_data::EditorData::new(palette, dimensions, opaques),
            properties_panel: level_panel::PropertiesPanel::new(),
            world_panel: world_panel::WorldPanel::new(),
            tile_panel: tile_panel::TilePanel::new(),
            bg_panel: bgpanel::BgPanel::new(),
            game_config_panel: game_config_panel::GameConfigPanel::new(),
            render_state,
            file_receiver,
            _debouncer: debouncer,
            watch_timer: std::time::Duration::from_secs(0),
        }
    }
}

impl eframe::App for LevelEditorApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        self.watch_timer += self.editor_data.dt;
        if self.watch_timer.as_secs() >= 1 {
            self.watch_timer = std::time::Duration::from_secs(0);
            match self.file_receiver.recv_timeout(std::time::Duration::from_nanos(1)) {
                Ok(res) => {
                    match res {
                        Ok(events) => {
                            for event in events {
                                log::info!("file changed: {}", event.path.to_string_lossy());
                                if event.path.ends_with(self.editor_data.level_gfx_file.as_str()) &&
                                    std::path::Path::new(("./".to_string() + self.editor_data.level_gfx_file.as_str()).as_str()).exists() {
                                    log::info!("hot-reloading GRAPHICS");
                                    let (palette, dimensions, opaques) = common_gfx::GfxCommonResources::insert_resource(&self.render_state, self.editor_data.level_gfx_file.as_str());
                                    self.editor_data.palette = palette;
                                    self.editor_data.gfx_size = dimensions;
                                    self.editor_data.opaques = opaques;
                                }
                            }
                        },
                        Err(e) => log::error!("hot-reloading error: {:?}", e),
                    }
                },
                Err(_) => {},
            }
        }

        ctx.request_repaint();
        self.editor_data.update_delta_time();
        egui::SidePanel::right("editor_options").resizable(true).max_width(500.0).show(ctx, |ui| {
            let mut force_gfx_reload = false;
            level_panel::show_metadata_panel(&mut self.world_panel, &mut self.editor_data, &mut self.mode, &mut self.level_data, ui, &mut self.bg_panel, &mut self.game_config_panel, &mut force_gfx_reload);
            if force_gfx_reload {
                log::info!("loading GRAPHICS");
                let (palette, dimensions, opaques) = common_gfx::GfxCommonResources::insert_resource(&self.render_state, self.editor_data.level_gfx_file.as_str());
                self.editor_data.palette = palette;
                self.editor_data.gfx_size = dimensions;
                self.editor_data.opaques = opaques;
            }
        });
        egui::SidePanel::left("editor_properties").resizable(true).max_width(500.0).show(ctx, |ui| {
            self.properties_panel.show_propeties_panel(&mut self.editor_data, &mut self.mode, &mut self.level_data, ui, &mut self.world_panel, &mut self.game_config_panel);
        });
        egui::TopBottomPanel::bottom("info_log").resizable(true).default_height(100.0).min_height(50.0).show(ctx, |ui| {
            logger::show_logs(&self.logs, ui);
        });
        egui::TopBottomPanel::top("info_bar").resizable(false).show(ctx, |ui| {
            ui.label(self.editor_data.info_bar.as_str());
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
                EditorMode::GameConfig => {
                    self.game_config_panel.update(ui, &mut self.editor_data);
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
