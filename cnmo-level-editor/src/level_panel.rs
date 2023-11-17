use cnmo_parse::lparse::level_data;
use cnmo_parse::lparse::level_data::cnmb_types::BackgroundImage;
use cnmo_parse::lparse::level_data::cnms_types::item_type::ItemType;
use cnmo_parse::lparse::level_data::cnms_types::wobj_type::{WobjType, CustomizableMovingPlatformType};
use cnmo_parse::lparse::level_data::cnms_types::{Spawner, SpawnerMode};
use eframe::egui;
use std::env;

use crate::editor_data::{EditorData, Tool};
use crate::game_config_panel::GameConfigPanel;
use crate::tile_viewer;
use crate::world_panel::WorldPanel;

crate::create_instance_resource!(LevelIconPreviewSpriteInstances);

pub fn show_metadata_panel(
    world_panel: &mut crate::world_panel::WorldPanel,
    editor_data: &mut EditorData,
    editor_mode: &mut super::EditorMode,
    level_data: &mut level_data::LevelData,
    ui: &mut egui::Ui,
    bg_panel: &mut crate::bgpanel::BgPanel,
    cfg_panel: &mut crate::game_config_panel::GameConfigPanel,
) {
    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
        ui.heading("File");
    });
    let text_response = ui.text_edit_singleline(&mut editor_data.level_file_name);
    if text_response.gained_focus() {
        editor_data.editing_text = Some(text_response.id);
    } else if text_response.lost_focus() && editor_data.editing_text == Some(text_response.id) {
        editor_data.editing_text = None;
    }
    text_response.on_hover_text("The level filename (without the extension)");
    if ui.button("New Level").clicked() {
        *level_data =
            level_data::LevelData::from_version(1).expect("Can't create version 1 level type?");
        log::info!("Created a new level!");
        editor_data.reset_selected_tiles();
        editor_data.level_file_name = "newlvl".to_string();
    }
    if ui.button("Load Level").clicked() {
        // let path = rfd::FileDialog::new()
        //     .set_directory("./")
        //     .set_title("Load an editor project file")
        //     .add_filter("Level Editor Files", &["json"])
        //     .pick_file();
        let path = Some(
            std::path::Path::new("levels/").join(editor_data.level_file_name.clone() + ".json"),
        );
        if let Some(path) = path {
            let file = std::fs::File::open(path);
            if let Ok(file) = file {
                if let Ok(data) = serde_json::from_reader(file) {
                    *level_data = data;
                    editor_data.reset_selected_tiles();
                    editor_data.cells_history =
                        vec![(level_data.cells.clone(), level_data.spawners.clone())];
                    log::info!("Loaded the level!");
                } else {
                    log::error!("Invalid editor project file!");
                }
            } else {
                log::warn!("Can't open the file!");
            }
        } else {
            log::warn!("File open dialog didn't return path!");
        }
    }
    if ui.button("Save Level").clicked() || (ui.ctx().input().key_pressed(egui::Key::S) && (ui.ctx().input().modifiers.mac_cmd || ui.ctx().input().modifiers.ctrl)) {
        // let path = rfd::FileDialog::new()
        //     .set_directory("./")
        //     .set_title("Save an editor project file")
        //     .add_filter("Level Editor Files", &["json"])
        //     .save_file();
        let path = Some(
            std::path::Path::new("levels/").join(editor_data.level_file_name.clone() + ".json"),
        );
        if let Some(path) = path {
            let file = std::fs::File::create(path);
            if let Ok(file) = file {
                let _ = serde_json::to_writer_pretty(file, level_data);
                log::info!("Saved the level!");
            } else {
                log::error!("Can't open the file for writing!");
            }
        } else {
            log::warn!("File open dialog didn't return path!");
        }
    }
    ui.label("");
    if ui.button("Decompile Level").clicked() {
        let paths = rfd::FileDialog::new()
            .set_directory("./")
            .set_title("Load an original cnm lparse level file")
            .add_filter("CNM Level Files", &["cnmb", "cnms"])
            .pick_files();
        if let Some(paths) = paths {
            let (mut cnmb, mut cnms) = (None, None);
            for path in paths.iter() {
                let ext = match path.extension() {
                    Some(ext) => ext.to_string_lossy().to_string(),
                    None => continue,
                };

                if ext == "cnmb" {
                    cnmb = Some(cnmo_parse::lparse::LParse::from_file(path));
                } else if ext == "cnms" {
                    cnms = Some(cnmo_parse::lparse::LParse::from_file(path));
                }
            }

            if let (Some(Ok(cnmb)), Some(Ok(cnms))) = (cnmb, cnms) {
                let result = level_data::LevelData::from_lparse(&cnmb, &cnms, false);
                match result {
                    Ok(data) => {
                        *level_data = data;
                        editor_data.reset_selected_tiles();
                        editor_data.cells_history =
                            vec![(level_data.cells.clone(), level_data.spawners.clone())];
                        log::info!("Successfully decompiled the level files!");
                    }
                    Err(err) => {
                        log::error!("Can't open cnm lparse files {:?}", paths);
                        log::error!("Error: {}", err);
                    }
                }
            } else {
                log::warn!("Can't open cnm lparse files {:?}", paths);
            }
        } else {
            log::warn!("File open dialog didn't return path!");
        }
    }
    let mut compile = || {
        // let cnmb_path = rfd::FileDialog::new()
        //     .set_directory("./")
        //     .set_title("Save an original cnm level file")
        //     .add_filter("CNMB Level Files", &["cnmb"])
        //     .save_file();
        let cnmb_path = Some(
            std::path::Path::new("levels/").join(editor_data.level_file_name.clone() + ".cnmb"),
        );
        let cnms_path = Some(
            std::path::Path::new("levels/").join(editor_data.level_file_name.clone() + ".cnms"),
        );
        // let cnms_path = if let Some(_) = cnmb_path {
        //     rfd::FileDialog::new()
        //         .set_directory("./")
        //         .set_title("Save an original cnm level file")
        //         .add_filter("CNMS Level Files", &["cnms"])
        //         .save_file()
        // } else {
        //     None
        // };
        if let (Some(cnmb_path), Some(cnms_path)) = (cnmb_path, cnms_path) {
            log::info!(
                "Compiling to {} and {}",
                cnmb_path.as_os_str().to_string_lossy(),
                cnms_path.as_os_str().to_string_lossy()
            );
            let cnmb = cnmo_parse::lparse::LParse::new(1);
            let cnms = cnmo_parse::lparse::LParse::new(1);
            if let (Ok(mut cnmb), Ok(mut cnms)) = (cnmb, cnms) {
                level_data.save(&mut cnmb, &mut cnms);
                editor_data.reset_selected_tiles();
                match (cnmb.save_to_file(cnmb_path), cnms.save_to_file(cnms_path)) {
                    (Ok(_), Ok(_)) => log::info!("Successfully compiled the level file!"),
                    _ => log::warn!(
                        "Can't compile the level file (maybe couldn't open it for writing?)!"
                    ),
                };
            } else {
                log::warn!("Can't create lparse version number 1");
            }
        } else {
            log::warn!("Can't open cnm lparse files");
        }
    };
    if ui.button("Compile Level").clicked() || (ui.ctx().input().key_pressed(egui::Key::E) && (ui.ctx().input().modifiers.mac_cmd || ui.ctx().input().modifiers.ctrl)) {
        compile();
    }
    ui.label("");
    if ui.button("Play Test!").clicked() {
        compile();
        let target_exes = match env::consts::OS {
            "windows" => vec![
                "cnmonline.exe",
                "build/cnmonline.exe",
                "build/CNMONLIN.exe",
                "CNMONLIN.exe",
                "CNMONLN.exe",
            ],
            _ => vec!["cnmonline", "build/cnmonline"],
        };
        for path in target_exes {
            log::info!(
                "running: {:?}",
                std::env::current_dir()
                    .expect("Need permission on current dir")
                    .join(path)
            );
            match std::process::Command::new(
                std::env::current_dir()
                    .expect("Need permission on current dir")
                    .join(path),
            ).args(["-editorwarp", "-level", ("levels/".to_string() + &editor_data.level_file_name).as_str()])
            .spawn()
            {
                Ok(_) => log::info!("Starting playtest session"),
                Err(_) => log::info!("Couldn't start {}", path),
            }
        }
    }
    ui.separator();
    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
        ui.heading("Metadata");
    });
    egui::Grid::new("level_metadata").num_columns(2).show(ui, |ui| {
        ui.label("Level Title: ");
        let text_response = ui.text_edit_singleline(&mut level_data.metadata.title);
        if text_response.gained_focus() {
            editor_data.editing_text = Some(text_response.id);
        } else if text_response.lost_focus() && editor_data.editing_text == Some(text_response.id) {
            editor_data.editing_text = None;
        }
        ui.end_row();
        {
            let mut level_name = level_data.metadata.subtitle.as_ref().unwrap_or(&"".to_string()).clone();
            ui.label("Level Subtitle: ");
            let text_response = ui.text_edit_singleline(&mut level_name);
            if text_response.gained_focus() {
                editor_data.editing_text = Some(text_response.id);
            } else if text_response.lost_focus() && editor_data.editing_text == Some(text_response.id) {
                editor_data.editing_text = None;
            }

            if text_response.changed() {
                level_data.metadata.subtitle = Some(level_name);
            }
            ui.end_row();
        }
        {
            ui.label("Level Icon X").on_hover_text("What 32x32 X tile the icon starts at in GFX.bmp\nIcons are 3x2 tiles big (96 by 32 pixels)");
            ui.label("Level Icon Y").on_hover_text("What 32x32 Y tile the icon starts at in GFX.bmp\nIcons are 3x2 tiles big (96 by 32 pixels)");
            ui.end_row();
            let loc = &mut level_data.metadata.preview_loc;
            ui.add(egui::DragValue::new(&mut loc.0).clamp_range(0..=editor_data.gfx_size.0 / 32 - 3));
            ui.add(egui::DragValue::new(&mut loc.1).clamp_range(0..=editor_data.gfx_size.1 / 32 - 2));
            ui.end_row();
            let (rect, _) = ui.allocate_exact_size(egui::Vec2::new(96.0, 64.0), egui::Sense::focusable_noninteractive());
            crate::instanced_sprites::InstancedSprites::new()
                .with_camera(crate::camera::Camera::new().with_projection(96.0, 64.0, None, true))
                .with_sprites(vec![crate::instanced_sprites::Sprite::new((0.0, 0.0, 0.0), (96.0, 64.0), (loc.0 as f32 * 32.0, loc.1 as f32 * 32.0, 96.0, 64.0))])
                .paint::<LevelIconPreviewSpriteInstances>(ui, rect);
            ui.end_row();
        }
        {
            let diff = &mut level_data.metadata.difficulty_rating;
            ui.label("Level Difficulty: ");
            egui::ComboBox::new("level_diff", "")
                .selected_text(diff.to_string_pretty())
                .show_ui(ui, |ui| {
                ui.selectable_value(diff, level_data::DifficultyRating::Tutorial, level_data::DifficultyRating::Tutorial.to_string_pretty());
                ui.selectable_value(diff, level_data::DifficultyRating::ReallyEasy, level_data::DifficultyRating::ReallyEasy.to_string_pretty());
                ui.selectable_value(diff, level_data::DifficultyRating::Easy, level_data::DifficultyRating::Easy.to_string_pretty());
                ui.selectable_value(diff, level_data::DifficultyRating::Normal, level_data::DifficultyRating::Normal.to_string_pretty());
                ui.selectable_value(diff, level_data::DifficultyRating::KindaHard, level_data::DifficultyRating::KindaHard.to_string_pretty());
                ui.selectable_value(diff, level_data::DifficultyRating::Hard, level_data::DifficultyRating::Hard.to_string_pretty());
                ui.selectable_value(diff, level_data::DifficultyRating::Ultra, level_data::DifficultyRating::Ultra.to_string_pretty());
                ui.selectable_value(diff, level_data::DifficultyRating::Extreme, level_data::DifficultyRating::Extreme.to_string_pretty());
                ui.selectable_value(diff, level_data::DifficultyRating::Dealth, level_data::DifficultyRating::Dealth.to_string_pretty());
                ui.selectable_value(diff, level_data::DifficultyRating::UltraDeath, level_data::DifficultyRating::UltraDeath.to_string_pretty());
            });
            ui.end_row();
        }
        {
            let level_type = &mut level_data.metadata.level_type;
            ui.label("Level Type: ");
            egui::ComboBox::new("level_type", "")
                .selected_text(level_type.to_string_pretty())
                .show_ui(ui, |ui| {
                ui.selectable_value(level_type, level_data::LevelType::Normal, level_data::LevelType::Normal.to_string_pretty());
                ui.selectable_value(level_type, level_data::LevelType::Secret, level_data::LevelType::Secret.to_string_pretty());
                ui.selectable_value(level_type, level_data::LevelType::Unlockable, level_data::LevelType::Unlockable.to_string_pretty());
                ui.selectable_value(level_type, level_data::LevelType::Unranked, level_data::LevelType::Unranked.to_string_pretty());
            });
            ui.end_row();
        }
        ui.label("Editor Mode: ");
        egui::ComboBox::new("editor_mode", "")
            .selected_text(format!("{:?}", editor_mode))
            .show_ui(ui, |ui| {
            if ui.selectable_value(editor_mode, super::EditorMode::Level, "Level").clicked() {
                world_panel.editing_background = false;
                editor_data.reset_selected_tiles();
            }
            if ui.selectable_value(editor_mode, super::EditorMode::Background, "Background").clicked() {
                world_panel.editing_background = true;
                editor_data.reset_selected_tiles();
                for pos in world_panel.background_pos.iter_mut() {
                    *pos = cgmath::vec2(0.0, 0.0);
                }
            }
            if ui.selectable_value(editor_mode, super::EditorMode::Tile, "Tile").clicked() {
                editor_data.reset_selected_tiles();
                editor_data.cells_history = vec![];
            }
            if ui.selectable_value(editor_mode, super::EditorMode::GameConfig, "Game Config").clicked() {
                editor_data.reset_selected_tiles();
                editor_data.cells_history = vec![];
            }
        });
        ui.end_row();

        if ui.ctx().input().key_pressed(egui::Key::T) && (ui.ctx().input().modifiers.ctrl || ui.ctx().input().modifiers.mac_cmd) {
            *editor_mode = super::EditorMode::Tile;
            editor_data.reset_selected_tiles();
            editor_data.editing_text = None;
        }
        if ui.ctx().input().key_pressed(egui::Key::B) && (ui.ctx().input().modifiers.ctrl || ui.ctx().input().modifiers.mac_cmd) {
            *editor_mode = super::EditorMode::Background;
            world_panel.editing_background = true;
            editor_data.reset_selected_tiles();
            editor_data.editing_text = None;
        }
        if ui.ctx().input().key_pressed(egui::Key::L) && (ui.ctx().input().modifiers.ctrl || ui.ctx().input().modifiers.mac_cmd) {
            *editor_mode = super::EditorMode::Level;
            world_panel.editing_background = false;
            editor_data.reset_selected_tiles();
            editor_data.editing_text = None;
        }
        if ui.ctx().input().key_pressed(egui::Key::G) && (ui.ctx().input().modifiers.ctrl || ui.ctx().input().modifiers.mac_cmd) {
            *editor_mode = super::EditorMode::GameConfig;
            world_panel.editing_background = false;
            editor_data.reset_selected_tiles();
            editor_data.editing_text = None;
        }
    });
    ui.separator();
    let show_background_list = |ui: &mut egui::Ui, world_panel: &mut WorldPanel| {
        let (mut start, mut end) = (
            *world_panel.background_range.start(),
            *world_panel.background_range.end(),
        );
        ui.label("Show Background Layers");
        ui.horizontal(|ui| {
            ui.add(egui::DragValue::new(&mut start).clamp_range(0..=31));
            ui.label("to");
            ui.add(egui::DragValue::new(&mut end).clamp_range(0..=31));
        });
        if start > end {
            start = end;
        }
        if end < start {
            end = start;
        }
        world_panel.background_range = start..=end;
    };
    match editor_mode {
        super::EditorMode::Level => {
            if ui
                .selectable_label(world_panel.gray_other, "Gray alternate tile layer")
                .clicked()
            {
                world_panel.gray_other = !world_panel.gray_other;
            }
            if ui
                .selectable_label(world_panel.show_grid, "Show grid")
                .clicked()
                || (ui.ctx().input().key_pressed(egui::Key::G) && editor_data.editing_text == None)
            {
                world_panel.show_grid = !world_panel.show_grid;
            }
            if matches!(editor_data.tool, Tool::Spawners) {
                egui::ComboBox::new("spawner_grid_size", "Spawner Grid Size")
                    .selected_text(if editor_data.spawner_grid_size < 1.0 {
                        "no grid".to_string()
                    } else {
                        editor_data.spawner_grid_size.to_string()
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut editor_data.spawner_grid_size, 1.0, "no grid");
                        ui.selectable_value(&mut editor_data.spawner_grid_size, 4.0, "4 pixels");
                        ui.selectable_value(&mut editor_data.spawner_grid_size, 8.0, "8 pixels");
                        ui.selectable_value(&mut editor_data.spawner_grid_size, 16.0, "half tile");
                        ui.selectable_value(&mut editor_data.spawner_grid_size, 32.0, "full tile");
                        ui.selectable_value(&mut editor_data.spawner_grid_size, 64.0, "2 tiles");
                    });
            }
            if ui
                .selectable_label(
                    world_panel.show_original_screen_size,
                    "Show CNM screen size",
                )
                .clicked()
            {
                world_panel.show_original_screen_size = !world_panel.show_original_screen_size;
            }
            ui.separator();
            show_background_list(ui, world_panel);
            ui.label("Tools");
            if ui
                .selectable_label(matches!(editor_data.tool, Tool::Brush), "Brush (B)")
                .clicked()
                || (ui.ctx().input().key_pressed(egui::Key::B) && editor_data.editing_text == None)
            {
                editor_data.tool = Tool::Brush;
            }
            if ui
                .selectable_label(matches!(editor_data.tool, Tool::Light), "Light (T)")
                .clicked()
                || (ui.ctx().input().key_pressed(egui::Key::T) && editor_data.editing_text == None)
            {
                editor_data.tool = Tool::Light;
            }
            if ui
                .selectable_label(matches!(editor_data.tool, Tool::Eraser), "Eraser (E)")
                .clicked()
                || (ui.ctx().input().key_pressed(egui::Key::E) && !(ui.ctx().input().modifiers.mac_cmd || ui.ctx().input().modifiers.ctrl) && editor_data.editing_text == None)
            {
                editor_data.tool = Tool::Eraser;
            }
            if ui
                .selectable_label(matches!(editor_data.tool, Tool::Fill), "Fill (F)")
                .clicked()
                || (ui.ctx().input().key_pressed(egui::Key::F) && editor_data.editing_text == None)
            {
                editor_data.tool = Tool::Fill;
            }
            if ui
                .selectable_label(
                    matches!(editor_data.tool, Tool::TilePicker),
                    "Tile Picker (R)",
                )
                .clicked()
                || (ui.ctx().input().key_pressed(egui::Key::R) && editor_data.editing_text == None)
            {
                editor_data.tool = Tool::TilePicker;
            }
            if ui
                .selectable_label(matches!(editor_data.tool, Tool::Spawners), "Spawners (S)")
                .clicked()
                || (ui.ctx().input().key_pressed(egui::Key::S) && !(ui.ctx().input().modifiers.mac_cmd || ui.ctx().input().modifiers.ctrl) && editor_data.editing_text == None)
            {
                editor_data.tool = Tool::Spawners;
            }
        }
        super::EditorMode::Background => {
            show_background_list(ui, world_panel);
            if ui
                .selectable_label(
                    editor_data.gray_out_background,
                    "Gray out non-selected background layers",
                )
                .clicked()
            {
                editor_data.gray_out_background = !editor_data.gray_out_background;
            }
            if editor_data.selecting_background_image {
                ui.horizontal(|ui| {
                    ui.label("Image Picking Grid Size");
                    ui.add(egui::DragValue::new(&mut bg_panel.grid_size).clamp_range(1..=128));
                });
            }
        }
        super::EditorMode::Tile => {}
        super::EditorMode::GameConfig => {
            if ui.button("Save Game Config").clicked() {
                let _ = std::fs::copy("audio.cnma", "audio.cnma.backup");
                match editor_data.game_config_file.save("audio.cnma") {
                    Ok(_) => log::info!("Successfully saved the game config file!"),
                    Err(err) => log::error!("Couldn't save config due to {}", err),
                };
            }
            if ui.button("Restore Last Load").clicked() {
                match cnmo_parse::cnma::Cnma::from_file("audio.cnma.backup") {
                    Ok(file) => {
                        editor_data.game_config_file = file;
                        log::info!("Successfully reloaded audio.cnma.backup!");
                    }
                    Err(_) => match cnmo_parse::cnma::Cnma::from_file("audio.cnma") {
                        Ok(file) => {
                            editor_data.game_config_file = file;
                            log::info!("Successfully reloaded audio.cnma!");
                        }
                        Err(err) => log::error!("Couldn't load config due to {}", err),
                    },
                };
            }
            ui.label("");
            ui.label("");
            if cfg_panel.preview_gfx {
                if ui.button("Close preview").clicked() {
                    cfg_panel.preview_gfx = false;
                }
            } else {
                if ui.button("Open GFX.BMP Preivew").clicked() {
                    cfg_panel.preview_gfx = true;
                }
            }
        }
    }
}

pub struct PropertiesPanel {
    tile_viewer: tile_viewer::TileViewer,
    dragging_bg: usize,
    dragging_bg_source: Option<usize>,
}

impl PropertiesPanel {
    pub fn new() -> Self {
        Self {
            tile_viewer: tile_viewer::TileViewer::new(None, Some(50.0), None, false),
            dragging_bg: 0,
            dragging_bg_source: None,
        }
    }

    pub fn show_propeties_panel(
        &mut self,
        editor_data: &mut EditorData,
        editor_mode: &mut super::EditorMode,
        level_data: &mut level_data::LevelData,
        ui: &mut egui::Ui,
        world_panel: &mut WorldPanel,
        game_config_panel: &mut GameConfigPanel,
    ) {
        match editor_mode {
            &mut super::EditorMode::Level => {
                self.show_level_panel(editor_data, level_data, ui, world_panel)
            }
            &mut super::EditorMode::Background => {
                self.show_background_panel(editor_data, level_data, ui, world_panel)
            }
            &mut super::EditorMode::Tile => self.show_tile_panel(editor_data, level_data, ui),
            &mut super::EditorMode::GameConfig => {
                self.show_game_config(editor_data, ui, game_config_panel)
            }
        }
    }

    fn show_game_config(
        &mut self,
        editor_data: &mut EditorData,
        ui: &mut egui::Ui,
        panel: &mut GameConfigPanel,
    ) {
        if panel.preview_gfx {
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.heading("Previewing...");
            });
            return;
        }
        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
            ui.heading("Sections");
        });
        ui.separator();
        for (idx, mode) in editor_data.game_config_file.modes.iter().enumerate() {
            if self.dragging_bg == idx
                && Some(self.dragging_bg) != self.dragging_bg_source
                && self.dragging_bg_source != None
            {
                ui.label("Move here");
            } else {
                let response = ui.selectable_label(
                    panel
                        .selected_mode
                        .unwrap_or(editor_data.game_config_file.modes.len())
                        == idx,
                    get_cnma_mode_name(mode),
                );
                if response.clicked() {
                    panel.selected_mode = Some(idx);
                }
                if ui.rect_contains_pointer(response.rect) {
                    if response.ctx.input().pointer.primary_clicked() {
                        self.dragging_bg_source = Some(idx);
                    }
                    self.dragging_bg = idx;
                }
            }
        }

        if let Some(src) = self.dragging_bg_source {
            if self.dragging_bg != src {
                egui::Area::new("config_mode_dragging")
                    .interactable(false)
                    .fixed_pos(ui.ctx().pointer_interact_pos().unwrap_or_default())
                    .show(ui.ctx(), |ui| {
                        ui.label(get_cnma_mode_name(&editor_data.game_config_file.modes[src]));
                    });
            }
        }
        if ui.ctx().input().pointer.any_released() {
            if let Some(src) = self.dragging_bg_source {
                let temp = editor_data.game_config_file.modes.remove(src);
                if self.dragging_bg <= src {
                    editor_data
                        .game_config_file
                        .modes
                        .insert(self.dragging_bg, temp);
                } else {
                    editor_data
                        .game_config_file
                        .modes
                        .insert(self.dragging_bg - 1, temp);
                }
            }
            self.dragging_bg = 0;
            self.dragging_bg_source = None;
        }

        egui::ComboBox::new("game_config_mode_add_combo_box", "")
            .selected_text("Add A New Config Section")
            .show_ui(ui, |ui| {
                let mut show_button = |mode: cnmo_parse::cnma::Mode| {
                    if ui.button(get_cnma_mode_name(&mode)).clicked() {
                        panel.selected_mode = Some(editor_data.game_config_file.modes.len());
                        editor_data.game_config_file.modes.push(mode);
                    }
                };
                show_button(cnmo_parse::cnma::Mode::SoundIds(vec![]));
                show_button(cnmo_parse::cnma::Mode::MusicIds(vec![]));
                show_button(cnmo_parse::cnma::Mode::LevelSelectOrder(vec![]));
                show_button(cnmo_parse::cnma::Mode::MaxPowerDef(Default::default()));
                show_button(cnmo_parse::cnma::Mode::LuaAutorunCode(String::new()));
                show_button(cnmo_parse::cnma::Mode::MusicVolumeOverride);
            });
    }

    fn show_level_panel(
        &mut self,
        editor_data: &mut EditorData,
        level_data: &mut level_data::LevelData,
        ui: &mut egui::Ui,
        world_panel: &mut WorldPanel,
    ) {
        if matches!(editor_data.tool, Tool::Light) {
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.heading("Light Edit Mode");
            });
            ui.horizontal(|ui| {
                ui.label("Color Level").on_hover_text("0 => White, 3 => Normal, 7 => Black");

                egui::ComboBox::new("light_combo_box", "")
                    .selected_text(editor_data.light_tool_level.to_string())
                    .show_ui(ui, |ui| {
                        for l in 0..=level_data::consts::LIGHT_BLACK {
                            ui.selectable_value(
                                &mut editor_data.light_tool_level,
                                l,
                                format!("{l}"),
                            );
                        }
                    });
            });
        } else if !matches!(editor_data.tool, Tool::Spawners) {
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.heading("Tiles");
            });
            ui.separator();
            ui.horizontal(|ui| {
                let response = ui.button(if editor_data.foreground_placing {
                    "Foreground"
                } else {
                    "Background"
                });
                if response.clicked() || (ui.ctx().input().key_pressed(egui::Key::Q) && editor_data.editing_text == None) {
                    editor_data.foreground_placing = !editor_data.foreground_placing;
                    //editor_data.light_placing = None;
                }
                response.on_hover_text("Press (Q) to switch between foreground and background");
            });
            self.tile_viewer.edit_tiles = false;
            self.tile_viewer.max_height = Some(ui.available_height() / 2.4);
            self.tile_viewer.show(ui, level_data, editor_data);
        } else {
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.heading("Spawner Properties");
            });
            ui.separator();
            if let Some(idx) = editor_data.selected_spawner {
                let spawner = &mut level_data.spawners[idx];

                egui::Grid::new("spawner_properties_grid").num_columns(2).striped(true).show(ui, |ui| {
                    ui.label("Spawning Mode: ");
                    egui::ComboBox::new("spawning_mode_combobox", "")
                        .selected_text(get_spawner_mode_name(&spawner.spawning_criteria.mode))
                        .show_ui(ui, |ui| {
                        ui.selectable_value(&mut spawner.spawning_criteria.mode, SpawnerMode::MultiAndSingleplayer, get_spawner_mode_name(&SpawnerMode::MultiAndSingleplayer));
                        ui.selectable_value(&mut spawner.spawning_criteria.mode, SpawnerMode::MultiplayerOnly, get_spawner_mode_name(&SpawnerMode::MultiplayerOnly));
                        ui.selectable_value(&mut spawner.spawning_criteria.mode, SpawnerMode::SingleplayerOnly, get_spawner_mode_name(&SpawnerMode::SingleplayerOnly));
                        ui.selectable_value(&mut spawner.spawning_criteria.mode, SpawnerMode::PlayerCountBased, get_spawner_mode_name(&SpawnerMode::PlayerCountBased)).on_hover_text_at_pointer(
                            "This will show in both single and multiplayer and will spawn an wobj for every\n
                            player in the server/game, this means the max spawns will be max_spawns*player_count"
                        );
                        ui.selectable_value(&mut spawner.spawning_criteria.mode, SpawnerMode::NoSpawn, get_spawner_mode_name(&SpawnerMode::NoSpawn));
                    });
                    ui.end_row();
                    ui.label("Delay between spawns: ").on_hover_text_at_pointer("Delay in seconds");
                    ui.add(egui::DragValue::new(&mut spawner.spawning_criteria.spawn_delay_secs)).on_hover_text_at_pointer("Delay in seconds");
                    ui.end_row();
                    ui.label("Number of maximum respawns: ");
                    ui.add(egui::DragValue::new(&mut spawner.spawning_criteria.max_concurrent_spawns));
                    ui.end_row();
                    ui.label("Drops item: ");
                    let dropped_item_response = ui.selectable_label(spawner.dropped_item != None, "Drops item");
                    if dropped_item_response.clicked() {
                        if spawner.dropped_item == None {
                            spawner.dropped_item = Some(ItemType::Shotgun);
                        } else {
                            spawner.dropped_item = None;
                        }
                    }
                    ui.end_row();
                    if let Some(item) = &mut spawner.dropped_item {
                        ui.label("Dropped Item: ");
                        show_item_combobox(item, ui);
                        ui.end_row();
                    }
                    ui.label("In group: ");
                    let group_response = ui.selectable_label(spawner.spawner_group != None, "Is Grouped");
                    if group_response.clicked() {
                        if spawner.spawner_group == None {
                            spawner.spawner_group = Some(0);
                        } else {
                            spawner.spawner_group = None;
                        }
                    }
                    ui.end_row();
                    if let Some(ref mut item) = &mut spawner.spawner_group {
                        ui.label("Group ID: ");
                        ui.add(egui::DragValue::new(item).clamp_range(0..=31));
                        ui.end_row();
                    }
                    show_spawner_properties(spawner, ui, editor_data, world_panel);
                });
            }
            ui.separator();
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.heading("Spawners");
            });
            ui.separator();
            egui::ScrollArea::vertical()
                .auto_shrink([true, true])
                .show(ui, |ui| {
                    use std::mem::discriminant;

                    egui::Grid::new("spawner_list_grid")
                        .striped(true)
                        .show(ui, |ui| {
                            for spawner_type in WobjIter::new() {
                                if discriminant(&WobjType::BanditGuy {
                                    speed: Default::default(),
                                }) == discriminant(&spawner_type)
                                {
                                    ui.end_row();
                                    ui.label("Enemies");
                                    ui.end_row();
                                    ui.end_row();
                                }
                                if discriminant(&WobjType::BreakablePlatform {
                                    time_till_fall: Default::default(),
                                }) == discriminant(&spawner_type)
                                {
                                    ui.end_row();
                                    ui.label("Environmental");
                                    ui.end_row();
                                    ui.end_row();
                                }
                                if discriminant(&WobjType::BackgroundSwitcher {
                                    shape: Default::default(),
                                    enabled_layers: Default::default(),
                                }) == discriminant(&spawner_type)
                                {
                                    ui.end_row();
                                    ui.label("Triggers");
                                    ui.end_row();
                                    ui.end_row();
                                }
                                if discriminant(&WobjType::DroppedItem {
                                    item: Default::default(),
                                }) == discriminant(&spawner_type)
                                {
                                    ui.end_row();
                                    ui.label("Collectables");
                                    ui.end_row();
                                    ui.end_row();
                                }

                                if ui
                                    .selectable_label(
                                        discriminant(
                                            &editor_data.spawner_template.type_data.clone(),
                                        ) == discriminant(&spawner_type),
                                        get_wobj_type_name(&spawner_type),
                                    )
                                    .clicked()
                                {
                                    editor_data.spawner_template.type_data = spawner_type;
                                }
                                ui.end_row();
                            }
                        });
                });
        }
    }

    fn show_background_panel(
        &mut self,
        editor_data: &mut EditorData,
        level_data: &mut level_data::LevelData,
        ui: &mut egui::Ui,
        world_panel: &mut WorldPanel,
    ) {
        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
            ui.heading("Background Properties");
        });
        egui::ScrollArea::new([false, true])
            .auto_shrink([false, true])
            .max_height(200.0)
            .show(ui, |ui| {
                for idx in 0..32 {
                    let response = if self.dragging_bg_source != None
                        && self.dragging_bg_source != Some(self.dragging_bg)
                        && self.dragging_bg_source == Some(idx)
                    {
                        ui.label("")
                    } else {
                        if self.dragging_bg_source != None
                            && self.dragging_bg_source != Some(self.dragging_bg)
                            && self.dragging_bg == idx
                        {
                            ui.label("Move Here");
                        }
                        ui.selectable_label(
                            editor_data.current_background == idx,
                            format!("Background Layer {}", idx),
                        )
                    };

                    let id = egui::Id::new("bglayer_item").with(idx);
                    if ui
                        .allocate_rect(response.rect, egui::Sense::hover())
                        .hovered()
                    {
                        self.dragging_bg = idx;
                    }
                    ui.interact(response.rect, id.clone(), egui::Sense::click_and_drag());
                    if ui.memory().is_being_dragged(id) {
                        self.dragging_bg_source = Some(idx);
                        editor_data.current_background = idx;
                    }
                }
            });

        if let Some(source_idx) = self.dragging_bg_source {
            if source_idx != self.dragging_bg {
                egui::Area::new("background_dragging")
                    .interactable(false)
                    .fixed_pos(ui.ctx().pointer_interact_pos().unwrap_or_default())
                    .show(ui.ctx(), |ui| {
                        ui.label(format!("Background Layer {source_idx}"));
                    });
            }
            if !ui
                .memory()
                .is_being_dragged(egui::Id::new("bglayer_item").with(source_idx))
            {
                level_data.background_layers.insert(
                    self.dragging_bg,
                    level_data.background_layers[source_idx].clone(),
                );
                if self.dragging_bg > source_idx {
                    level_data.background_layers.remove(source_idx);
                } else {
                    level_data.background_layers.remove(source_idx + 1);
                }
                self.dragging_bg_source = None;
            }
        }

        ui.separator();
        let layer = &mut level_data.background_layers[editor_data.current_background];
        egui::Grid::new("background_editor_grid")
            .striped(true)
            .num_columns(2)
            .show(ui, |ui| {
                ui.label("Origin: ")
                    .on_hover_text("Origin of the first background layer");
                ui.horizontal(|ui| {
                    ui.add(egui::DragValue::new(&mut layer.origin.0).speed(0.75));
                    ui.add(egui::DragValue::new(&mut layer.origin.1).speed(0.75));
                });
                ui.end_row();
                ui.label("Scroll Speed: ")
                    .on_hover_text("Bigger values make the layer scroll more slowly");
                ui.horizontal(|ui| {
                    ui.add(egui::DragValue::new(&mut layer.scroll_speed.0).speed(0.75));
                    ui.add(egui::DragValue::new(&mut layer.scroll_speed.1).speed(0.75));
                });
                ui.end_row();
                ui.label("Speed: ")
                    .on_hover_text("How many pixels the background scrolls per frame (30 fps)");
                ui.horizontal(|ui| {
                    if ui
                        .add(egui::DragValue::new(&mut layer.speed.0).speed(0.75))
                        .changed()
                    {
                        world_panel.background_pos[editor_data.current_background].x = 0.0;
                    }
                    if ui
                        .add(egui::DragValue::new(&mut layer.speed.1).speed(0.75))
                        .changed()
                    {
                        world_panel.background_pos[editor_data.current_background].y = 0.0;
                    }
                });
                ui.end_row();
                ui.label("Spacing: ")
                    .on_hover_text("How many pixels of spacing are between each background image");
                ui.horizontal(|ui| {
                    ui.add(egui::DragValue::new(&mut layer.spacing.0).speed(1.0));
                    ui.add(egui::DragValue::new(&mut layer.spacing.1).speed(1.0));
                });
                ui.end_row();
                ui.label("Transparency: ")
                    .on_hover_text("0 (fully opaque) to 7 (fully transparent)");
                ui.add(
                    egui::DragValue::new(&mut layer.transparency)
                        .speed(1.0)
                        .clamp_range(0..=7),
                );
                ui.end_row();
                ui.label("Repeat: ");
                if ui.selectable_label(layer.repeat_up, "Upwards").clicked() {
                    layer.repeat_up = !layer.repeat_up;
                }
                ui.end_row();
                ui.label("Repeat: ");
                if ui
                    .selectable_label(layer.repeat_down, "Downwards")
                    .clicked()
                {
                    layer.repeat_down = !layer.repeat_down;
                }
                ui.end_row();
                ui.label("Repeat: ");
                if ui
                    .selectable_label(layer.repeat_horizontally, "Horizontally")
                    .clicked()
                {
                    layer.repeat_horizontally = !layer.repeat_horizontally;
                }
                ui.end_row();
                ui.label("Infront of screen: ");
                if ui
                    .selectable_label(layer.in_foreground, "In Foreground")
                    .clicked()
                {
                    layer.in_foreground = !layer.in_foreground;
                }
                ui.end_row();
                ui.label("3D Top Width: ")
                    .on_hover_text("How many pixels wide is the top of the 3d projection");
                ui.add(egui::DragValue::new(&mut layer.top3d).speed(1.0));
                ui.end_row();
                ui.label("3D Bottom Width: ")
                    .on_hover_text("How many pixels wide is the bottom of the 3d projection");
                ui.add(egui::DragValue::new(&mut layer.bottom3d).speed(1.0));
                ui.end_row();
                ui.label("3D Height: ")
                    .on_hover_text("How high the 3d projection is");
                ui.add(egui::DragValue::new(&mut layer.height3d).speed(1.0));
                ui.end_row();
                ui.label("Image/Color: ");
                egui::ComboBox::new("background_image_chooser", "")
                    .selected_text(match layer.image {
                        BackgroundImage::Color(_) => "Whole Color",
                        BackgroundImage::Bitmap(_) => "Image",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut layer.image,
                            BackgroundImage::Color(0),
                            "Whole Color",
                        );
                        ui.selectable_value(
                            &mut layer.image,
                            BackgroundImage::Bitmap(cnmo_parse::Rect {
                                x: 0,
                                y: 0,
                                w: 0,
                                h: 0,
                            }),
                            "Image",
                        );
                    });
                ui.end_row();
                match &mut layer.image {
                    BackgroundImage::Color(color) => {
                        ui.label("Edit Color");
                        ui.add(egui::DragValue::new(color).clamp_range(0..=255));
                        ui.end_row();
                        if ui.button("Pick Color").clicked() {
                            editor_data.selecting_background_color = true;
                            editor_data.selecting_background_image = false;
                        }
                    }
                    BackgroundImage::Bitmap(rect) => {
                        ui.label("Rect X: ")
                            .on_hover_text("Start of the background image in GFX.BMP");
                        ui.horizontal(|ui| {
                            ui.add(egui::DragValue::new(&mut rect.x).speed(1.0));
                        });
                        ui.end_row();
                        ui.label("Rect Y: ")
                            .on_hover_text("Start of the background image in GFX.BMP");
                        ui.horizontal(|ui| {
                            ui.add(egui::DragValue::new(&mut rect.y).speed(1.0));
                        });
                        ui.end_row();
                        ui.label("Rect W: ")
                            .on_hover_text("Size of the background image in GFX.BMP");
                        ui.horizontal(|ui| {
                            ui.add(egui::DragValue::new(&mut rect.w).speed(1.0));
                        });
                        ui.end_row();
                        ui.label("Rect H: ")
                            .on_hover_text("Size of the background image in GFX.BMP");
                        ui.horizontal(|ui| {
                            ui.add(egui::DragValue::new(&mut rect.h).speed(1.0));
                        });
                        ui.end_row();
                        if ui.button("Pick Image").clicked() {
                            editor_data.selecting_background_color = false;
                            editor_data.selecting_background_image = true;
                        }
                    }
                }
                ui.end_row();
            });
    }

    fn show_tile_panel(
        &mut self,
        editor_data: &mut EditorData,
        level_data: &mut level_data::LevelData,
        ui: &mut egui::Ui,
    ) {
        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
            ui.heading("Tiles");
        });
        ui.separator();
        if ui.button("New Tile").clicked() {
            level_data
                .tile_properties
                .push(level_data::cnmb_types::TileProperties::default());
            editor_data.selected_tiles = vec![level_data.tile_properties.len() - 1];
        }
        self.tile_viewer.edit_tiles = true;
        self.tile_viewer.max_height = None;
        self.tile_viewer.show(ui, level_data, editor_data);
    }
}

fn show_item_combobox(item: &mut ItemType, ui: &mut egui::Ui) {
    egui::ComboBox::new("item_combo_box", "")
        .selected_text(get_item_type_name(item))
        .show_ui(ui, |ui| {
            for item_type in ItemIter::new() {
                ui.selectable_value(item, item_type, get_item_type_name(&item_type));
            }
        });
}

fn show_spawner_properties(
    spawner: &mut Spawner,
    ui: &mut egui::Ui,
    editor_data: &mut EditorData,
    world_panel: &mut WorldPanel,
) {
    use cnmo_parse::lparse::level_data::cnms_types::wobj_type::{
        BackgroundSwitcherShape, KeyColor, PushZoneType, RockGuyType, RuneType, TtNodeType,
        TunesTriggerSize, UpgradeTriggerType,
    };
    match &mut spawner.type_data {
        &mut WobjType::Checkpoint {
            ref mut checkpoint_num,
        } => {
            ui.label("Checkpoint ID: ")
                .on_hover_text("What order the checkpoints go in");
            ui.add(egui::DragValue::new(checkpoint_num).clamp_range(0..=255));
            ui.end_row();
        }
        &mut WobjType::Teleport(ref mut teleport) => {
            ui.label("Name: ");
            let text_response = ui.text_edit_singleline(&mut teleport.name);
            if text_response.gained_focus() {
                editor_data.editing_text = Some(text_response.id);
            } else if text_response.lost_focus()
                && editor_data.editing_text == Some(text_response.id)
            {
                editor_data.editing_text = None;
            }
            ui.end_row();
            ui.label("Cost: ");
            ui.add(egui::DragValue::new(&mut teleport.cost));
            ui.end_row();
            ui.label("Location X: ");
            ui.add(egui::DragValue::new(&mut teleport.loc.0));
            ui.end_row();
            ui.label("Location Y: ");
            ui.add(egui::DragValue::new(&mut teleport.loc.1));
            ui.end_row();
            ui.label("");
            if ui.button("Center view on teleport location").clicked() {
                world_panel.camera.pos.x = teleport.loc.0;
                world_panel.camera.pos.y = teleport.loc.1;
            }
            ui.end_row();
            ui.label("");
            if ui.button("Center view back on teleport").clicked() {
                world_panel.camera.pos.x = spawner.pos.0;
                world_panel.camera.pos.y = spawner.pos.1;
            }
            ui.end_row();
            if editor_data.spawner_grid_size > 1.0 {
                ui.label("");
                if ui.button("Snap teleport location to grid").clicked() {
                    teleport.loc.0 = (teleport.loc.0 / editor_data.spawner_grid_size).round()
                        * editor_data.spawner_grid_size;
                    teleport.loc.1 = (teleport.loc.1 / editor_data.spawner_grid_size).round()
                        * editor_data.spawner_grid_size;
                }
                ui.end_row();
            }
        }
        &mut WobjType::TeleportArea1 {
            ref mut link_id,
            ref mut loc,
        } => {
            ui.label("Link ID: ");
            ui.add(egui::DragValue::new(link_id).clamp_range(0..=65535));
            ui.end_row();
            ui.label("Location X: ");
            ui.add(egui::DragValue::new(&mut loc.0));
            ui.end_row();
            ui.label("Location Y: ");
            ui.add(egui::DragValue::new(&mut loc.1));
            ui.end_row();
            ui.label("");
            if ui.button("Center view on teleport location").clicked() {
                world_panel.camera.pos.x = loc.0;
                world_panel.camera.pos.y = loc.1;
            }
            ui.end_row();
            ui.label("");
            if ui.button("Center view back on trigger area").clicked() {
                world_panel.camera.pos.x = spawner.pos.0;
                world_panel.camera.pos.y = spawner.pos.1;
            }
            ui.end_row();
            if editor_data.spawner_grid_size > 1.0 {
                ui.label("");
                if ui.button("Snap teleport location to grid").clicked() {
                    loc.0 = (loc.0 / editor_data.spawner_grid_size).round()
                        * editor_data.spawner_grid_size;
                    loc.1 = (loc.1 / editor_data.spawner_grid_size).round()
                        * editor_data.spawner_grid_size;
                }
                ui.end_row();
            }
        }
        &mut WobjType::TeleportArea2 {
            ref mut link_id,
            ref mut loc,
            ref mut start_activated,
            ref mut teleport_players,
        } => {
            ui.label("Link ID: ");
            ui.add(egui::DragValue::new(link_id).clamp_range(0..=65535));
            ui.end_row();
            ui.label("Location X: ");
            ui.add(egui::DragValue::new(&mut loc.0));
            ui.end_row();
            ui.label("Location Y: ");
            ui.add(egui::DragValue::new(&mut loc.1));
            ui.end_row();
            ui.label("");
            if ui.button("Center view on teleport location").clicked() {
                world_panel.camera.pos.x = loc.0;
                world_panel.camera.pos.y = loc.1;
            }
            ui.end_row();
            ui.label("");
            if ui.button("Center view back on trigger area").clicked() {
                world_panel.camera.pos.x = spawner.pos.0;
                world_panel.camera.pos.y = spawner.pos.1;
            }
            ui.end_row();
            if editor_data.spawner_grid_size > 1.0 {
                ui.label("");
                if ui.button("Snap teleport location to grid").clicked() {
                    loc.0 = (loc.0 / editor_data.spawner_grid_size).round()
                        * editor_data.spawner_grid_size;
                    loc.1 = (loc.1 / editor_data.spawner_grid_size).round()
                        * editor_data.spawner_grid_size;
                }
                ui.end_row();
            }
            if ui.selectable_label(*teleport_players, "Teleport Players").clicked() {
                *teleport_players = !*teleport_players;
            }
            if ui.selectable_label(*start_activated, "Start Activated").clicked() {
                *start_activated = !*start_activated;
            }
            ui.end_row();
        }
        &mut WobjType::TunesTrigger {
            ref mut size,
            ref mut music_id,
        } => {
            ui.label("Trigger Size: ");
            egui::ComboBox::new("trigger_size_combo_box", "")
                .selected_text(match size {
                    TunesTriggerSize::Small => "1x1",
                    TunesTriggerSize::Big => "2x2",
                    TunesTriggerSize::VeryBig => "3x3",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(size, TunesTriggerSize::Small, "1x1");
                    ui.selectable_value(size, TunesTriggerSize::Big, "2x2");
                    ui.selectable_value(size, TunesTriggerSize::VeryBig, "3x3");
                });
            ui.end_row();
            ui.label("Music ID: ");
            ui.add(egui::DragValue::new(music_id));
            ui.end_row();
        }
        &mut WobjType::TextSpawner {
            ref mut dialoge_box,
            ref mut despawn,
            ref mut text,
        } => {
            if ui
                .selectable_label(*dialoge_box, "Is dialoge box")
                .clicked()
            {
                *dialoge_box = !*dialoge_box;
            }
            if ui
                .selectable_label(*despawn, "Despawn after being seen")
                .clicked()
            {
                *despawn = !*despawn;
            }
            ui.end_row();
            ui.label("Lines");
            ui.end_row();
            let text_response = ui.text_edit_multiline(text);
            if text_response.gained_focus() {
                editor_data.editing_text = Some(text_response.id);
            } else if text_response.lost_focus()
                && editor_data.editing_text == Some(text_response.id)
            {
                editor_data.editing_text = None;
            }
        }
        &mut WobjType::BackgroundSwitcher {
            ref mut shape,
            ref mut enabled_layers,
        } => {
            ui.label("Enabled Layers");
            ui.end_row();
            ui.label("Start layer");
            ui.add(egui::DragValue::new(&mut enabled_layers.start).clamp_range(0..=32));
            ui.end_row();
            ui.label("End layer");
            ui.add(egui::DragValue::new(&mut enabled_layers.end).clamp_range(0..=32));
            ui.end_row();
            ui.label("Shape: ");
            egui::ComboBox::new("bgswitcher_shape", "")
                .selected_text(match shape {
                    BackgroundSwitcherShape::Horizontal => "Horizontal",
                    BackgroundSwitcherShape::Small => "Small",
                    BackgroundSwitcherShape::Vertical => "Vertical",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(shape, BackgroundSwitcherShape::Small, "Small");
                    ui.selectable_value(shape, BackgroundSwitcherShape::Horizontal, "Horizontal");
                    ui.selectable_value(shape, BackgroundSwitcherShape::Vertical, "Vertical");
                });
            ui.end_row();
            ui.label("");
            if ui.button("Test background layers").clicked() {
                world_panel.background_range =
                    enabled_layers.start as usize..=enabled_layers.end as usize;
            }
            ui.end_row();
        }
        &mut WobjType::DroppedItem { ref mut item } => {
            ui.label("Item: ");
            egui::ComboBox::new("item_type", "")
                .selected_text(get_item_type_name(item))
                .show_ui(ui, |ui| {
                    for item_type in ItemIter::new() {
                        ui.selectable_value(item, item_type, get_item_type_name(&item_type));
                    }
                });
        }
        &mut WobjType::TtNode { ref mut node_type } => {
            ui.label("Node Type: ");
            egui::ComboBox::new("ttnode_type", "")
                .selected_text(match node_type {
                    TtNodeType::ChaseTrigger => "Dead Brother",
                    TtNodeType::NormalTrigger => "Golden Brother",
                    TtNodeType::Waypoint(_) => "Waypoint",
                    TtNodeType::BozoWaypoint => "Bozo Waypoint",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(node_type, TtNodeType::ChaseTrigger, "Dead Brother");
                    ui.selectable_value(node_type, TtNodeType::NormalTrigger, "Golden Brother");
                    ui.selectable_value(
                        node_type,
                        TtNodeType::BozoWaypoint,
                        "Bozo Waypoint (32 max)",
                    );
                    ui.selectable_value(node_type, TtNodeType::Waypoint(0), "Waypoint");
                });
            ui.end_row();
            if let TtNodeType::Waypoint(id) = node_type {
                ui.label("Waypoint ID: ");
                ui.add(egui::DragValue::new(id).clamp_range(0..=127));
                ui.end_row();
            }
        }
        &mut WobjType::RotatingFireColunmPiece {
            ref mut origin_x,
            ref mut degrees_per_second,
        } => {
            ui.label("Origin X: ");
            ui.add(egui::DragValue::new(origin_x));
            ui.end_row();
            ui.label("Degrees Per Second: ");
            ui.add(egui::DragValue::new(degrees_per_second));
            ui.end_row();
        }
        &mut WobjType::MovingFire {
            ref mut vertical,
            ref mut dist,
            ref mut speed,
            ref mut despawn,
        } => {
            ui.label("");
            if ui.selectable_label(*vertical, "Is Vertical").clicked() {
                *vertical = !*vertical;
            }
            ui.end_row();
            ui.label("Distance: ");
            ui.add(egui::DragValue::new(dist));
            ui.end_row();
            ui.label("Speed: ");
            ui.add(egui::DragValue::new(speed).clamp_range(0.0..=256.0));
            ui.end_row();
            ui.label("Despawn: ");
            if ui.selectable_label(*despawn, "Despawn").clicked() {
                *despawn = !*despawn;
            }
            ui.end_row();
        }
        &mut WobjType::PushZone {
            ref mut push_zone_type,
            ref mut push_speed,
        } => {
            ui.label("Zone Type: ");
            egui::ComboBox::new("push_zone_type", "")
                .selected_text(match push_zone_type {
                    PushZoneType::Horizontal => "Horizontal",
                    PushZoneType::Vertical => "Vertical",
                    PushZoneType::HorizontalSmall => "Small",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(push_zone_type, PushZoneType::Horizontal, "Horizontal");
                    ui.selectable_value(push_zone_type, PushZoneType::Vertical, "Vertical");
                    ui.selectable_value(push_zone_type, PushZoneType::HorizontalSmall, "Small");
                });
            ui.end_row();
            ui.label("Speed: ");
            ui.add(egui::DragValue::new(push_speed));
            ui.end_row();
        }
        &mut WobjType::VerticalWindZone {
            ref mut acceleration,
        } => {
            ui.label("Gravity (acceleration): ");
            ui.add(egui::DragValue::new(acceleration));
            ui.end_row();
        }
        &mut WobjType::SuperDragonLandingZone {
            ref mut waypoint_id,
        } => {
            ui.label("Waypoint ID: ");
            ui.add(egui::DragValue::new(waypoint_id).clamp_range(0..=15));
            ui.end_row();
        }
        &mut WobjType::Jumpthrough { ref mut big } => {
            ui.label("");
            if ui.selectable_label(*big, "Big").clicked() {
                *big = !*big;
            }
            ui.end_row();
        }
        &mut WobjType::HealthSetTrigger {
            ref mut target_health,
        } => {
            ui.label("Target HP: ");
            ui.add(egui::DragValue::new(target_health));
            ui.end_row();
        }
        &mut WobjType::GraphicsChangeTrigger { ref mut gfx_file } => {
            ui.label("Graphics File: ");
            let text_response = ui.text_edit_singleline(gfx_file);
            if text_response.gained_focus() {
                editor_data.editing_text = Some(text_response.id);
            } else if text_response.lost_focus()
                && editor_data.editing_text == Some(text_response.id)
            {
                editor_data.editing_text = None;
            }
            ui.end_row();
        }
        &mut WobjType::BossBarInfo { ref mut boss_name } => {
            ui.label("Boss Name: ");
            let text_response = ui.text_edit_singleline(boss_name);
            if text_response.gained_focus() {
                editor_data.editing_text = Some(text_response.id);
            } else if text_response.lost_focus()
                && editor_data.editing_text == Some(text_response.id)
            {
                editor_data.editing_text = None;
            }
            ui.end_row();
        }
        &mut WobjType::BgSpeed {
            ref mut vertical_axis,
            ref mut layer,
            ref mut speed,
        } => {
            ui.label("");
            if ui.selectable_label(*vertical_axis, "Is Y Speed").clicked() {
                *vertical_axis = !*vertical_axis;
            }
            ui.end_row();
            ui.label("Layer ID: ");
            ui.add(egui::DragValue::new(layer).clamp_range(0..=31));
            ui.end_row();
            ui.label("Speed (pixels per frame): ");
            ui.add(egui::DragValue::new(speed));
            ui.end_row();
        }
        &mut WobjType::BgTransparency {
            ref mut layer,
            ref mut transparency,
        } => {
            ui.label("Layer ID: ");
            ui.add(egui::DragValue::new(layer).clamp_range(0..=31));
            ui.end_row();
            ui.label("Transparency: ");
            ui.add(egui::DragValue::new(transparency).clamp_range(0..=7));
            ui.end_row();
        }
        &mut WobjType::TeleportTrigger1 {
            ref mut link_id,
            ref mut delay_secs,
        } => {
            ui.label("Link ID: ");
            ui.add(egui::DragValue::new(link_id).clamp_range(0..=255));
            ui.end_row();
            ui.label("Trigger Delay (secs): ");
            ui.add(egui::DragValue::new(delay_secs));
            ui.end_row();
        }
        &mut WobjType::SfxPoint { ref mut sound_id } => {
            ui.label("Sound Effect ID: ");
            ui.add(egui::DragValue::new(sound_id).clamp_range(0..=255));
            ui.end_row();
        }
        &mut WobjType::BreakableWall {
            ref mut skin_id,
            ref mut health,
        } => {
            ui.label("Wall HP: ");
            ui.add(egui::DragValue::new(health));
            ui.end_row();
            if let Some(id) = skin_id {
                ui.label("Skin ID: ");
                ui.add(egui::DragValue::new(id).clamp_range(0..=5));
                ui.end_row();
                ui.label("");
                if ui.button("Don't use custom skin").clicked() {
                    *skin_id = None;
                }
                ui.end_row();
            } else {
                ui.label("");
                if ui.button("Use custom skin").clicked() {
                    *skin_id = Some(0);
                }
            }
        }
        &mut WobjType::MovingPlatform {
            ref mut vertical,
            ref mut dist,
            ref mut speed,
        } => {
            ui.label("");
            if ui.selectable_label(*vertical, "Is Vertical").clicked() {
                *vertical = !*vertical;
            }
            ui.end_row();
            ui.label("Distance: ");
            ui.add(egui::DragValue::new(dist));
            ui.end_row();
            ui.label("Speed: ");
            ui.add(egui::DragValue::new(speed));
            ui.end_row();
        }
        &mut WobjType::DisapearingPlatform {
            ref mut time_on,
            ref mut time_off,
            ref mut starts_on,
        } => {
            ui.label("");
            if ui.selectable_label(*starts_on, "Starts on").clicked() {
                *starts_on = !*starts_on;
            }
            ui.end_row();
            ui.label("Time On");
            ui.add(egui::DragValue::new(time_on));
            ui.end_row();
            ui.label("Time Off");
            ui.add(egui::DragValue::new(time_off));
            ui.end_row();
        }
        &mut WobjType::SpringBoard {
            ref mut jump_velocity,
        } => {
            ui.label("Jump velocity");
            ui.add(egui::DragValue::new(jump_velocity));
            ui.end_row();
        }
        &mut WobjType::BreakablePlatform {
            ref mut time_till_fall,
        } => {
            ui.label("Time till falling: ");
            ui.add(egui::DragValue::new(time_till_fall));
            ui.end_row();
        }
        &mut WobjType::CustomizeableMoveablePlatform {
            ref mut bitmap_x32,
            ref mut target_relative,
            ref mut speed,
            ref mut start_paused,
            ref mut ty,
        } => {
            ui.label("Bitmap Tile X: ");
            ui.add(egui::DragValue::new(&mut bitmap_x32.0));
            ui.end_row();
            ui.label("Bitmap Tile Y: ");
            ui.add(egui::DragValue::new(&mut bitmap_x32.1));
            ui.end_row();
            ui.label("Target Relative X: ");
            ui.add(egui::DragValue::new(&mut target_relative.0));
            ui.end_row();
            ui.label("Target Relative Y: ");
            ui.add(egui::DragValue::new(&mut target_relative.1));
            ui.end_row();
            ui.label("Speed (pixels per frame): ");
            ui.add(egui::DragValue::new(speed));
            ui.end_row();
            if ui
                .selectable_label(*start_paused, "Starts Paused")
                .clicked()
            {
                *start_paused = !*start_paused;
            }
            //if ui.selectable_label(*one_way, "Only 1 Way").clicked() {
            //    *one_way = !*one_way;
            //}
            ui.end_row();
            ui.label("Type: ").on_hover_text("You can set it to despawn once it hits its target destination, or to only go one way");
            egui::ComboBox::new("cmpf_combo_box", "")
                .selected_text(match ty {
                    CustomizableMovingPlatformType::Normal => "Normal",
                    CustomizableMovingPlatformType::OneWay => "One Way",
                    CustomizableMovingPlatformType::Despawn => "Despawn",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(ty, CustomizableMovingPlatformType::Normal, "Normal");
                    ui.selectable_value(ty, CustomizableMovingPlatformType::OneWay, "One Way");
                    ui.selectable_value(ty, CustomizableMovingPlatformType::Despawn, "Despawn");
                });
            ui.end_row();
            let frames_in_dir = ((target_relative.0.powi(2) + target_relative.1.powi(2)).sqrt() / *speed).ceil();
            ui.label("Seconds/Frames Till Turn:");
            ui.label(format!("{}/{}", frames_in_dir / 30.0, frames_in_dir as i32));
            ui.end_row();
        }
        &mut WobjType::LockedBlock {
            ref mut color,
            ref mut consume_key,
        } => {
            ui.label("Color: ");
            egui::ComboBox::new("color_combo_box", "")
                .selected_text(match color {
                    KeyColor::Red => "Red",
                    KeyColor::Green => "Green",
                    KeyColor::Blue => "Blue",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(color, KeyColor::Red, "Red");
                    ui.selectable_value(color, KeyColor::Green, "Green");
                    ui.selectable_value(color, KeyColor::Blue, "Blue");
                });
            ui.end_row();
            ui.label("");
            if ui.selectable_label(*consume_key, "Consume key").clicked() {
                *consume_key = !*consume_key;
            }
            ui.end_row();
        }
        &mut WobjType::Vortex {
            ref mut attract_enemies,
        } => {
            ui.label("");
            if ui
                .selectable_label(*attract_enemies, "Attract Enemies")
                .clicked()
            {
                *attract_enemies = !*attract_enemies;
            }
            ui.end_row();
        }
        &mut WobjType::WandRune { ref mut rune_type } => {
            ui.label("Rune Type: ");
            egui::ComboBox::new("rune_type_combo_box", "")
                .selected_text(match rune_type {
                    RuneType::Air => "Air",
                    RuneType::Fire => "Fire",
                    RuneType::Ice => "Ice",
                    RuneType::Lightning => "Lightning",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(rune_type, RuneType::Air, "Air");
                    ui.selectable_value(rune_type, RuneType::Fire, "Fire");
                    ui.selectable_value(rune_type, RuneType::Ice, "Ice");
                    ui.selectable_value(rune_type, RuneType::Lightning, "Lightning");
                });
            ui.end_row();
        }
        &mut WobjType::UpgradeTrigger {
            ref mut trigger_type,
        } => {
            ui.label("Upgrade Type: ");
            egui::ComboBox::new("upgrade_type_combo_box", "")
                .selected_text(match trigger_type {
                    UpgradeTriggerType::Wings => "Wings",
                    UpgradeTriggerType::CrystalWings => "Crystal Wings",
                    UpgradeTriggerType::DeephausBoots => "Deephaus Boots",
                    UpgradeTriggerType::Vortex => "Vortex",
                    UpgradeTriggerType::None => "Remove Upgrades",
                    UpgradeTriggerType::MaxPowerRune { .. } => "Max Power Rune",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(trigger_type, UpgradeTriggerType::Wings, "Wings");
                    ui.selectable_value(
                        trigger_type,
                        UpgradeTriggerType::CrystalWings,
                        "Crystal Wings",
                    );
                    ui.selectable_value(
                        trigger_type,
                        UpgradeTriggerType::DeephausBoots,
                        "Deephaus Boots",
                    );
                    ui.selectable_value(trigger_type, UpgradeTriggerType::Vortex, "Vortex");
                    ui.selectable_value(trigger_type, UpgradeTriggerType::None, "Remove Upgrades");
                    ui.selectable_value(
                        trigger_type,
                        UpgradeTriggerType::MaxPowerRune {
                            skin_power_override: None,
                        },
                        "Max Power Rune",
                    );
                });
            ui.end_row();
            if let UpgradeTriggerType::MaxPowerRune {
                skin_power_override,
            } = trigger_type
            {
                if let Some(id) = skin_power_override {
                    ui.label("Skin ID: ");
                    ui.add(egui::DragValue::new(id).clamp_range(0..=31));
                    ui.end_row();
                    ui.label("");
                    if ui.button("Don't override skin").clicked() {
                        *skin_power_override = None;
                    }
                    ui.end_row();
                } else {
                    ui.label("");
                    if ui.button("Override skin").clicked() {
                        *skin_power_override = Some(0);
                    }
                    ui.end_row();
                }
            }
        }
        &mut WobjType::Slime { ref mut flying } => {
            ui.label("");
            if ui.selectable_label(*flying, "Flying Type").clicked() {
                *flying = !*flying;
            }
            ui.end_row();
        }
        &mut WobjType::Heavy {
            ref mut speed,
            ref mut face_left,
        } => {
            ui.label("");
            if ui.selectable_label(*face_left, "Face left").clicked() {
                *face_left = !*face_left;
            }
            ui.end_row();
            ui.label("Speed");
            ui.add(egui::DragValue::new(speed));
            ui.end_row();
        }
        &mut WobjType::Dragon { ref mut space_skin } => {
            ui.label("");
            if ui.selectable_label(*space_skin, "Use space skin").clicked() {
                *space_skin = !*space_skin;
            }
            ui.end_row();
        }
        &mut WobjType::Bozo { ref mut mark_ii } => {
            ui.label("");
            if ui
                .selectable_label(*mark_ii, "Mark II Version Boss")
                .clicked()
            {
                *mark_ii = !*mark_ii;
            }
            ui.end_row();
        }
        &mut WobjType::LavaMonster { ref mut face_left } => {
            ui.label("");
            if ui.selectable_label(*face_left, "Face left").clicked() {
                *face_left = !*face_left;
            }
            ui.end_row();
        }
        &mut WobjType::TtMinion { ref mut small } => {
            ui.label("");
            if ui.selectable_label(*small, "Small").clicked() {
                *small = !*small;
            }
            ui.end_row();
        }
        &mut WobjType::MegaFish {
            ref mut water_level,
            ref mut swimming_speed,
        } => {
            ui.label("Water level");
            ui.add(egui::DragValue::new(water_level));
            ui.end_row();
            ui.label("Speed");
            ui.add(egui::DragValue::new(swimming_speed));
            ui.end_row();
        }
        &mut WobjType::LavaDragonHead {
            ref mut len,
            ref mut health,
        } => {
            ui.label("Number of segments");
            ui.add(egui::DragValue::new(len).clamp_range(2..=31));
            ui.end_row();
            ui.label("Health");
            ui.add(egui::DragValue::new(health));
            ui.end_row();
        }
        &mut WobjType::BanditGuy { ref mut speed }
        | &mut WobjType::BozoLaserMinion { ref mut speed }
        | &mut WobjType::SpiderWalker { ref mut speed }
        | &mut WobjType::TtBoss { ref mut speed } => {
            ui.label("Speed");
            ui.add(egui::DragValue::new(speed));
            ui.end_row();
        }
        &mut WobjType::BozoPin {
            ref mut flying_speed,
        } => {
            ui.label("Flying Speed");
            ui.add(egui::DragValue::new(flying_speed));
            ui.end_row();
        }
        &mut WobjType::EaterBug {
            ref mut pop_up_speed,
        } => {
            ui.label("Pop Down Speed");
            ui.add(egui::DragValue::new(pop_up_speed));
            ui.end_row();
        }
        &mut WobjType::SuperDragon {
            ref mut waypoint_id,
        } => {
            ui.label("Landing waypoint");
            ui.add(egui::DragValue::new(waypoint_id).clamp_range(0..=15));
            ui.end_row();
        }
        &mut WobjType::RockGuy {
            ref mut rock_guy_type,
        } => {
            ui.label("Size: ");
            egui::ComboBox::new("rock_guy_type_combo_box", "")
                .selected_text(match rock_guy_type {
                    RockGuyType::Medium => "Meduim",
                    RockGuyType::Small1 => "Small",
                    RockGuyType::Small2 { .. } => "Very Small",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(rock_guy_type, RockGuyType::Medium, "Meduim");
                    ui.selectable_value(rock_guy_type, RockGuyType::Small1, "Small");
                    ui.selectable_value(
                        rock_guy_type,
                        RockGuyType::Small2 { face_left: false },
                        "Very Small",
                    );
                });
            ui.end_row();
            if let RockGuyType::Small2 { face_left } = rock_guy_type {
                ui.label("");
                if ui.selectable_label(*face_left, "Face Left").clicked() {
                    *face_left = !*face_left;
                }
                ui.end_row();
            }
        }
        &mut WobjType::Lua {
            ref mut lua_wobj_type,
        } => {
            ui.label("Lua Type ID");
            ui.add(egui::DragValue::new(lua_wobj_type).clamp_range(0..=15));
            ui.end_row();
        }
        &mut WobjType::GravityTrigger { ref mut gravity } => {
            ui.label("Gravity")
                .on_hover_text("0.5 is the default gravity");
            ui.add(egui::DragValue::new(gravity));
            ui.end_row();
        }
        &mut WobjType::FinishTrigger { ref mut next_level, ref mut extra_unlocked_level, ref mut is_secret } => {
            ui.label("Next Level: ").on_hover_text("The name of the next level file in the levels/ directiory without the extension");
            let text_response = ui.text_edit_singleline(next_level);
            if text_response.gained_focus() {
                editor_data.editing_text = Some(text_response.id);
            } else if text_response.lost_focus()
                && editor_data.editing_text == Some(text_response.id)
            {
                editor_data.editing_text = None;
            }
            ui.end_row();
            ui.label("Is Secret Exit: ");
            if ui.selectable_label(*is_secret, "Secret").clicked() {
                *is_secret = !*is_secret;
            }
            ui.end_row();
            ui.label("Unlocks Extra Level: ");
            let extra_level_response = ui.selectable_label(extra_unlocked_level != &None, "Yes");
            if extra_level_response.clicked() {
                if extra_unlocked_level == &None {
                    *extra_unlocked_level = Some("level_name".to_string());
                } else {
                    *extra_unlocked_level = None;
                }
            }
            ui.end_row();
            if let Some(extra_level) = extra_unlocked_level {
                ui.label("Unlocks: ").on_hover_text("The name of the next level file in the levels/ directiory without the extension");
                let text_response = ui.text_edit_singleline(extra_level);
                if text_response.gained_focus() {
                    editor_data.editing_text = Some(text_response.id);
                } else if text_response.lost_focus()
                    && editor_data.editing_text == Some(text_response.id)
                {
                    editor_data.editing_text = None;
                }
                ui.end_row();
            }
        }
        &mut WobjType::SkinUnlock { ref mut id } => {
            ui.label("Skin ID");
            ui.add(egui::DragValue::new(id).clamp_range(0..=10));
            ui.end_row();
        }
        &mut WobjType::CoolPlatform { ref mut time_off_before, ref mut time_on, ref mut time_off_after } => {
            ui.label("Time off before");
            ui.add(egui::DragValue::new(time_off_before)).on_hover_text("This is in frames. CNM Online runs at 30 fps");
            ui.end_row();
            ui.label("Time on");
            ui.add(egui::DragValue::new(time_on)).on_hover_text("This is in frames. CNM Online runs at 30 fps");
            ui.end_row();
            ui.label("Time off after");
            ui.add(egui::DragValue::new(time_off_after)).on_hover_text("This is in frames. CNM Online runs at 30 fps");
            ui.end_row();
        }
        _ => {}
    }
}

fn get_cnma_mode_name(mode: &cnmo_parse::cnma::Mode) -> &str {
    use cnmo_parse::cnma::Mode;
    match mode {
        &Mode::SoundIds(_) => "Sound IDS",
        &Mode::MusicIds(_) => "Music IDS",
        &Mode::LevelSelectOrder(_) => "Level Order",
        &Mode::LuaAutorunCode(_) => "LUA Code",
        &Mode::MaxPowerDef(_) => "Max Power Abilities",
        &Mode::MusicVolumeOverride => "Deprecated (Music Volume Override)",
    }
}

fn get_spawner_mode_name(mode: &SpawnerMode) -> &str {
    match mode {
        &SpawnerMode::MultiAndSingleplayer => "Multi and Singleplayer",
        &SpawnerMode::MultiplayerOnly => "Multiplayer Only",
        &SpawnerMode::SingleplayerOnly => "Singleplayer Only",
        &SpawnerMode::PlayerCountBased => "Player Count Based",
        &SpawnerMode::NoSpawn => "Don't Spawn In Level",
    }
}

struct ItemIter {
    index: u8,
}

impl ItemIter {
    fn new() -> Self {
        Self { index: 0 }
    }
}

impl Iterator for ItemIter {
    type Item = ItemType;

    fn next(&mut self) -> Option<Self::Item> {
        use ItemType::*;

        let index = self.index;
        self.index += 1;
        match index {
            0 => Some(Shotgun),
            1 => Some(Knife),
            2 => Some(Apple),
            3 => Some(Cake),
            4 => Some(StrengthPotion),
            5 => Some(SpeedPotion),
            6 => Some(JumpPotion),
            7 => Some(Sword),
            8 => Some(HealthPotion),
            9 => Some(Sniper),
            10 => Some(Money50),
            11 => Some(Money100),
            12 => Some(Money500),
            13 => Some(Cheeseburger),
            14 => Some(GoldenAxe),
            15 => Some(UnboundWand),
            16 => Some(FireWand),
            17 => Some(IceWand),
            18 => Some(AirWand),
            19 => Some(LightningWand),
            20 => Some(GoldenShotgun),
            21 => Some(LaserRifle),
            22 => Some(RocketLauncher),
            23 => Some(FirePotion),
            24 => Some(Minigun),
            25 => Some(MegaPotion),
            26 => Some(UltraMegaPotion),
            27 => Some(Awp),
            28 => Some(Flamethrower),
            29 => Some(PoisionusStrengthPotion),
            30 => Some(PoisionusSpeedPotion),
            31 => Some(PoisionusJumpPotion),
            32 => Some(Beastchurger),
            33 => Some(UltraSword),
            34 => Some(HeavyHammer),
            35 => Some(FissionGun),
            36 => Some(KeyRed),
            37 => Some(KeyGreen),
            38 => Some(KeyBlue),
            39 => Some(ExtraLifeJuice),
            40 => Some(Wrench),
            _ => None,
        }
    }
}

fn get_item_type_name(item_type: &ItemType) -> &str {
    use ItemType::*;

    match item_type {
        &Shotgun => "Shotgun",
        &Knife => "Knife",
        &Apple => "Apple",
        &Cake => "Cake",
        &StrengthPotion => "Strength Potion",
        &SpeedPotion => "Speed Potion",
        &JumpPotion => "Jump Potion",
        &Sword => "Sword",
        &HealthPotion => "Health Potion",
        &Sniper => "Sniper",
        &Money50 => "50 Money",
        &Money100 => "100 Money",
        &Money500 => "500 Money",
        &Cheeseburger => "Cheeseburger",
        &GoldenAxe => "Golden Axe",
        &UnboundWand => "Unbound Wand",
        &FireWand => "Fire Wand",
        &IceWand => "Ice Wand",
        &AirWand => "Air Wand",
        &LightningWand => "Lightning Wand",
        &GoldenShotgun => "Golden Shotgun",
        &LaserRifle => "Laser Rifle",
        &RocketLauncher => "Rocket Launcher",
        &FirePotion => "Fire Potion",
        &Minigun => "Minigun",
        &MegaPotion => "Mega Potion",
        &UltraMegaPotion => "Ultra Mega Potion",
        &Awp => "Awp",
        &Flamethrower => "Flamethrower",
        &PoisionusStrengthPotion => "Bad Strength Potion",
        &PoisionusSpeedPotion => "Bad Speed Potion",
        &PoisionusJumpPotion => "Bad Jump Potion",
        &Beastchurger => "Beastchurger",
        &UltraSword => "Ultra Sword",
        &HeavyHammer => "Heavy Hammer",
        &FissionGun => "Fission Gun",
        &KeyRed => "Key Red",
        &KeyGreen => "Key Green",
        &KeyBlue => "Key Blue",
        &ExtraLifeJuice => "1-Up Juice",
        &Wrench => "Wrench",
    }
}

struct WobjIter {
    index: u8,
}

impl WobjIter {
    fn new() -> Self {
        Self { index: 0 }
    }
}

impl Iterator for WobjIter {
    type Item = WobjType;

    fn next(&mut self) -> Option<Self::Item> {
        use WobjType::*;

        let index = self.index;
        self.index += 1;
        match index {
            // Enemies
            00 => Some(BanditGuy {
                speed: Default::default(),
            }),
            01 => Some(Bozo {
                mark_ii: Default::default(),
            }),
            02 => Some(BozoLaserMinion {
                speed: Default::default(),
            }),
            03 => Some(BozoPin {
                flying_speed: Default::default(),
            }),
            04 => Some(Dragon {
                space_skin: Default::default(),
            }),
            05 => Some(Heavy {
                speed: Default::default(),
                face_left: Default::default(),
            }),
            06 => Some(EaterBug {
                pop_up_speed: Default::default(),
            }),
            07 => Some(KamakaziSlime),
            08 => Some(LavaDragonHead {
                len: Default::default(),
                health: Default::default(),
            }),
            09 => Some(LavaMonster {
                face_left: Default::default(),
            }),
            10 => Some(Lua {
                lua_wobj_type: Default::default(),
            }),
            11 => Some(MegaFish {
                water_level: Default::default(),
                swimming_speed: Default::default(),
            }),
            12 => Some(RockGuy {
                rock_guy_type: Default::default(),
            }),
            13 => Some(RockGuySlider),
            14 => Some(RockGuySmasher),
            15 => Some(SilverSlime),
            16 => Some(Slime { flying: false }),
            17 => Some(SlimeWalker),
            18 => Some(SpiderWalker {
                speed: Default::default(),
            }),
            19 => Some(SpikeGuy),
            20 => Some(SuperDragon {
                waypoint_id: Default::default(),
            }),
            21 => Some(Supervirus),
            22 => Some(TtBoss {
                speed: Default::default(),
            }),
            23 => Some(TtMinion {
                small: Default::default(),
            }),
            24 => Some(Wolf),

            // Environmental
            25 => Some(BreakablePlatform {
                time_till_fall: Default::default(),
            }),
            26 => Some(BreakableWall {
                skin_id: Default::default(),
                health: Default::default(),
            }),
            27 => Some(CustomizeableMoveablePlatform {
                bitmap_x32: Default::default(),
                target_relative: Default::default(),
                speed: Default::default(),
                start_paused: Default::default(),
                ty: Default::default(),
            }),
            28 => Some(DisapearingPlatform {
                time_on: Default::default(),
                time_off: Default::default(),
                starts_on: Default::default(),
            }),
            29 => Some(Jumpthrough {
                big: Default::default(),
            }),
            30 => Some(LockedBlock {
                color: Default::default(),
                consume_key: Default::default(),
            }),
            31 => Some(MovingFire {
                vertical: Default::default(),
                dist: Default::default(),
                speed: Default::default(),
                despawn: Default::default(),
            }),
            32 => Some(MovingPlatform {
                vertical: Default::default(),
                dist: Default::default(),
                speed: Default::default(),
            }),
            33 => Some(PushZone {
                push_zone_type: Default::default(),
                push_speed: Default::default(),
            }),
            34 => Some(RotatingFireColunmPiece {
                origin_x: Default::default(),
                degrees_per_second: Default::default(),
            }),
            35 => Some(SpikeTrap),
            36 => Some(SpringBoard {
                jump_velocity: Default::default(),
            }),
            37 => Some(SuperDragonLandingZone {
                waypoint_id: Default::default(),
            }),
            38 => Some(VerticalWindZone {
                acceleration: Default::default(),
            }),
            39 => Some(Vortex {
                attract_enemies: Default::default(),
            }),
            40 => Some(CoolPlatform {
                time_off_before: Default::default(),
                time_on: Default::default(),
                time_off_after: Default::default(),
            }),

            // Triggers
            41 => Some(BackgroundSwitcher {
                shape: Default::default(),
                enabled_layers: Default::default(),
            }),
            42 => Some(BgSpeed {
                vertical_axis: Default::default(),
                layer: Default::default(),
                speed: Default::default(),
            }),
            43 => Some(BgTransparency {
                layer: Default::default(),
                transparency: Default::default(),
            }),
            44 => Some(BossBarInfo {
                boss_name: Default::default(),
            }),
            45 => Some(Checkpoint {
                checkpoint_num: Default::default(),
            }),
            46 => Some(GraphicsChangeTrigger {
                gfx_file: Default::default(),
            }),
            47 => Some(HealthSetTrigger {
                target_health: Default::default(),
            }),
            48 => Some(PlayerSpawn),
            49 => Some(SfxPoint {
                sound_id: Default::default(),
            }),
            50 => Some(Teleport(Default::default())),
            51 => Some(TeleportArea1 {
                link_id: Default::default(),
                loc: Default::default(),
            }),
            52 => Some(TeleportArea2 {
                link_id: Default::default(),
                loc: Default::default(),
                teleport_players: true,
                start_activated: false,
            }),
            53 => Some(TeleportTrigger1 {
                link_id: Default::default(),
                delay_secs: Default::default(),
            }),
            54 => Some(TextSpawner {
                dialoge_box: Default::default(),
                despawn: false,
                text: Default::default(),
            }),
            55 => Some(TtNode {
                node_type: Default::default(),
            }),
            56 => Some(TunesTrigger {
                size: Default::default(),
                music_id: Default::default(),
            }),
            57 => Some(FinishTrigger {
                next_level: Default::default(),
                extra_unlocked_level: None,
                is_secret: false,
            }),
            58 => Some(GravityTrigger {
                gravity: Default::default(),
            }),

            // Collectables
            59 => Some(DroppedItem {
                item: Default::default(),
            }),
            60 => Some(UpgradeTrigger {
                trigger_type: Default::default(),
            }),
            61 => Some(WandRune {
                rune_type: Default::default(),
            }),
            62 => Some(SkinUnlock {
                id: Default::default(),
            }),
            _ => None,
        }
    }
}

fn get_wobj_type_name(wobj_type: &WobjType) -> &str {
    use WobjType::*;
    match &wobj_type {
        &Teleport(_) => "Teleport",
        &Slime { .. } => "Slime",
        &TunesTrigger { .. } => "Tunes Trigger",
        &PlayerSpawn => "Player Spawn",
        &TextSpawner { .. } => "Text Spawner",
        &MovingPlatform { .. } => "Moving Platform",
        &BreakableWall { .. } => "Breakable Wall",
        &BackgroundSwitcher { .. } => "Background Switcher",
        &DroppedItem { .. } => "Dropped Item",
        &WandRune { .. } => "Wand Rune",
        &Heavy { .. } => "Heavy",
        &Dragon { .. } => "Dragon",
        &BozoPin { .. } => "Bozopin",
        &Bozo { .. } => "Bozo",
        &SilverSlime => "Silver Slime",
        &LavaMonster { .. } => "Lava Monster",
        &TtMinion { .. } => "Tt Minion",
        &SlimeWalker => "Slime Walker",
        &MegaFish { .. } => "Mega Fish",
        &LavaDragonHead { .. } => "Lava Dragon Head",
        &TtNode { .. } => "Tt Node",
        &TtBoss { .. } => "Tt Boss",
        &EaterBug { .. } => "Eater Bug",
        &SpiderWalker { .. } => "Spider Walker",
        &SpikeTrap => "Spike Trap",
        &RotatingFireColunmPiece { .. } => "Rotating Fire Colunm Piece",
        &MovingFire { .. } => "Moving Fire",
        &SuperDragon { .. } => "Super Dragon",
        &SuperDragonLandingZone { .. } => "Super Dragon Landing Zone",
        &BozoLaserMinion { .. } => "Bozo Laser Minion",
        &Checkpoint { .. } => "Checkpoint",
        &SpikeGuy => "Spike Guy",
        &BanditGuy { .. } => "Bandit Guy",
        &PushZone { .. } => "Push Zone",
        &VerticalWindZone { .. } => "Vertical Wind Zone",
        &DisapearingPlatform { .. } => "Disapearing Platform",
        &KamakaziSlime => "Kamakazi Slime",
        &SpringBoard { .. } => "Spring Board",
        &Jumpthrough { .. } => "Jumpthrough",
        &BreakablePlatform { .. } => "Breakable Platform",
        &LockedBlock { .. } => "Locked Block",
        &RockGuy { .. } => "Rock Guy",
        &RockGuySlider => "Rock Guy Slider",
        &RockGuySmasher => "Rock Guy Smasher",
        &HealthSetTrigger { .. } => "Health Set Trigger",
        &Vortex { .. } => "Vortex",
        &CustomizeableMoveablePlatform { .. } => "Customizeable Moveable Platform",
        &GraphicsChangeTrigger { .. } => "Graphics Change Trigger",
        &BossBarInfo { .. } => "Boss Bar Info",
        &BgSpeed { .. } => "Bg Speed",
        &BgTransparency { .. } => "Bg Transparency",
        &TeleportTrigger1 { .. } => "Teleport Trigger",
        &TeleportArea1 { .. } => "Teleport Area",
        &SfxPoint { .. } => "Sfx Point",
        &Wolf => "Wolf",
        &Supervirus => "Supervirus",
        &Lua { .. } => "Lua",
        &UpgradeTrigger { .. } => "Upgrade",
        &FinishTrigger { .. } => "Finish Trigger",
        &GravityTrigger { .. } => "Gravity Trigger",
        &SkinUnlock { .. } => "Skin Unlock",
        &CoolPlatform { .. } => "Megaman Platform",
        &TeleportArea2 { .. } => "Teleport Area (Teleports Everything)",
    }
}
