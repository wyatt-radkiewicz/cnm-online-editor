use cnmo_parse::lparse::level_data::cnmb_types::BackgroundImage;
use eframe::egui;
use cnmo_parse::lparse::level_data;

use crate::tile_viewer;
use crate::editor_data::{
    EditorData,
    Tool,
};
use crate::world_panel::WorldPanel;

pub fn show_metadata_panel(world_panel: &mut crate::world_panel::WorldPanel, editor_data: &mut EditorData, editor_mode: &mut super::EditorMode, level_data: &mut level_data::LevelData, ui: &mut egui::Ui, bg_panel: &mut crate::bgpanel::BgPanel) {
    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
        ui.heading("File");
    });
    if ui.button("New Level").clicked() {
        *level_data = level_data::LevelData::from_version(1).expect("Can't create version 1 level type?");
        log::info!("Created a new level!");
        editor_data.reset_selected_tiles();
    }
    if ui.button("Load Level").clicked() {
        let path = rfd::FileDialog::new()
            .set_directory("./")
            .set_title("Load an editor project file")
            .add_filter("Level Editor Files", &["json"])
            .pick_file();
        if let Some(path) = path {
            let file = std::fs::File::open(path);
            if let Ok(file) = file {
                if let Ok(data) = serde_json::from_reader(file) {
                    *level_data = data;
                    editor_data.reset_selected_tiles();
                    editor_data.cells_history = vec![level_data.cells.clone()];
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
    if ui.button("Save Level").clicked() {
        let path = rfd::FileDialog::new()
            .set_directory("./")
            .set_title("Save an editor project file")
            .add_filter("Level Editor Files", &["json"])
            .save_file();
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
                        editor_data.cells_history = vec![level_data.cells.clone()];
                        log::info!("Successfully decompiled the level files!");
                    },
                    Err(err) => {
                        log::error!("Can't open cnm lparse files {:?}", paths);
                        log::error!("Error: {}", err);
                    },
                }
            } else {
                log::warn!("Can't open cnm lparse files {:?}", paths);
            }
        } else {
            log::warn!("File open dialog didn't return path!");
        }
    }
    if ui.button("Compile Level").clicked() {
        let cnmb_path = rfd::FileDialog::new()
            .set_directory("./")
            .set_title("Save an original cnm level file")
            .add_filter("CNMB Level Files", &["cnmb"])
            .save_file();
        let cnms_path = if let Some(_) = cnmb_path {
            rfd::FileDialog::new()
                .set_directory("./")
                .set_title("Save an original cnm level file")
                .add_filter("CNMS Level Files", &["cnms"])
                .save_file()
        } else {
            None
        };
        if let (Some(cnmb_path), Some(cnms_path)) = (cnmb_path, cnms_path) {
            log::info!("Compiling to {} and {}", cnmb_path.as_os_str().to_string_lossy(), cnms_path.as_os_str().to_string_lossy());
            let cnmb = cnmo_parse::lparse::LParse::new(1);
            let cnms = cnmo_parse::lparse::LParse::new(1);
            if let (Ok(mut cnmb), Ok(mut cnms)) = (cnmb, cnms) {
                level_data.save(&mut cnmb, &mut cnms);
                editor_data.reset_selected_tiles();
                match (cnmb.save_to_file(cnmb_path), cnms.save_to_file(cnms_path)) {
                    (Ok(_), Ok(_)) => log::info!("Successfully compiled the level file!"),
                    _ => log::warn!("Can't compile the level file (maybe couldn't open it for writing?)!"),
                };
            } else {
                log::warn!("Can't create lparse version number 1");
            }
        } else {
            log::warn!("Can't open cnm lparse files");
        }
    }
    ui.separator();
    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
        ui.heading("Metadata");
    });
    egui::Grid::new("level_metadata").num_columns(2).show(ui, |ui| {
        ui.label("Level Title: ");
        ui.text_edit_singleline(&mut level_data.metadata.title);
        ui.end_row();
        {
            let mut level_name = level_data.metadata.subtitle.as_ref().unwrap_or(&"".to_string()).clone();
            ui.label("Level Subtitle: ");
            if ui.text_edit_singleline(&mut level_name).changed() {
                level_data.metadata.subtitle = Some(level_name);
            }
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
        });
        ui.end_row();

        if ui.ctx().input().key_pressed(egui::Key::T) && ui.ctx().input().modifiers.ctrl {
            *editor_mode = super::EditorMode::Tile;
            editor_data.reset_selected_tiles();
        }
        if ui.ctx().input().key_pressed(egui::Key::B) && ui.ctx().input().modifiers.ctrl {
            *editor_mode = super::EditorMode::Background;
            world_panel.editing_background = true;
            editor_data.reset_selected_tiles();
        }
        if ui.ctx().input().key_pressed(egui::Key::L) && ui.ctx().input().modifiers.ctrl {
            *editor_mode = super::EditorMode::Level;
            world_panel.editing_background = false;
            editor_data.reset_selected_tiles();
        }
    });
    ui.separator();
    let show_background_list = |ui: &mut egui::Ui, world_panel: &mut WorldPanel| {
        let (mut start, mut end) = (*world_panel.background_range.start(), *world_panel.background_range.end());
        ui.label("Show Background Layers");
        ui.horizontal(|ui| {
            ui.add(egui::DragValue::new(&mut start).clamp_range(0..=31));
            ui.label("to");
            ui.add(egui::DragValue::new(&mut end).clamp_range(0..=31));
        });
        if start > end { start = end; }
        if end < start { end = start; }
        world_panel.background_range = start..=end;
    };
    match editor_mode {
        super::EditorMode::Level => {
            if ui.selectable_label(world_panel.gray_other, "Gray alternate tile layer").clicked() {
                world_panel.gray_other = !world_panel.gray_other;
            }
            if ui.selectable_label(world_panel.show_grid, "Show grid").clicked() || ui.ctx().input().key_pressed(egui::Key::G) {
                world_panel.show_grid = !world_panel.show_grid;
            }
            if ui.selectable_label(world_panel.show_original_screen_size, "Show CNM screen size").clicked() {
                world_panel.show_original_screen_size = !world_panel.show_original_screen_size;
            }
            ui.separator();
            show_background_list(ui, world_panel);
            ui.label("Tools");
            if ui.selectable_label(matches!(editor_data.tool, Tool::Brush), "Brush (B)").clicked() ||
                ui.ctx().input().key_pressed(egui::Key::B) {
                editor_data.tool = Tool::Brush;
            }
            if ui.selectable_label(matches!(editor_data.tool, Tool::Eraser), "Eraser (E)").clicked() ||
                ui.ctx().input().key_pressed(egui::Key::E) {
                editor_data.tool = Tool::Eraser;
            }
            if ui.selectable_label(matches!(editor_data.tool, Tool::Fill), "Fill (F)").clicked() ||
                ui.ctx().input().key_pressed(egui::Key::F) {
                editor_data.tool = Tool::Fill;
            }
            if ui.selectable_label(matches!(editor_data.tool, Tool::TilePicker), "Tile Picker (R)").clicked() ||
                ui.ctx().input().key_pressed(egui::Key::R) {
                editor_data.tool = Tool::TilePicker;
            }
            if ui.selectable_label(matches!(editor_data.tool, Tool::Spawners), "Spawners (S)").clicked() ||
                ui.ctx().input().key_pressed(egui::Key::S) {
                editor_data.tool = Tool::Spawners;
            }
        },
        super::EditorMode::Background => {
            show_background_list(ui, world_panel);
            if ui.selectable_label(editor_data.gray_out_background, "Gray out non-selected background layers").clicked() {
                editor_data.gray_out_background = !editor_data.gray_out_background;
            }
            if editor_data.selecting_background_image {
                ui.horizontal(|ui| {
                    ui.label("Image Picking Grid Size");
                    ui.add(egui::DragValue::new(&mut bg_panel.grid_size).clamp_range(1..=128));
                });
            }
        },
        super::EditorMode::Tile => {

        },
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

    pub fn show_propeties_panel(&mut self, editor_data: &mut EditorData, editor_mode: &mut super::EditorMode, level_data: &mut level_data::LevelData, ui: &mut egui::Ui, world_panel: &mut WorldPanel) {
        match editor_mode {
            &mut super::EditorMode::Level => self.show_level_panel(editor_data, level_data, ui),
            &mut super::EditorMode::Background => self.show_background_panel(editor_data, level_data, ui, world_panel),
            &mut super::EditorMode::Tile => self.show_tile_panel(editor_data, level_data, ui),
        }
    }
    
    fn show_level_panel(&mut self, editor_data: &mut EditorData, level_data: &mut level_data::LevelData, ui: &mut egui::Ui) {
        if !matches!(editor_data.tool, Tool::Spawners) {
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.heading("Tiles");
            });
            ui.separator();
            ui.horizontal(|ui| {
                let response = ui.button(if editor_data.foreground_placing { "Foreground" } else { "Background" });
                if response.clicked() || ui.ctx().input().key_pressed(egui::Key::Q) {
                    editor_data.foreground_placing = !editor_data.foreground_placing;
                    editor_data.light_placing = None;
                }
                response.on_hover_text("Press (Q) to switch between foreground and background");
                egui::ComboBox::new("light_combo_box", "")
                    .selected_text(format!("{}", if let Some(id) = editor_data.light_placing { id.to_string() } else { "***".to_string() }))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut editor_data.light_placing, None, "***");
                        for l in 0..level_data::consts::LIGHT_WHITE {
                            ui.selectable_value(&mut editor_data.light_placing, Some(l), format!("{l}"));
                        }
                });
            });
            self.tile_viewer.edit_tiles = false;
            self.tile_viewer.max_height = Some(ui.available_height() / 2.4);
            self.tile_viewer.show(ui, level_data, editor_data);
        } else {
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.heading("Spawner Properties");
            });
            ui.separator();
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.heading("Spawners");
            });
            ui.separator();
        }
    }
    
    fn show_background_panel(&mut self, editor_data: &mut EditorData, level_data: &mut level_data::LevelData, ui: &mut egui::Ui, world_panel: &mut WorldPanel) {
        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
            ui.heading("Background Properties");
        });
        egui::ScrollArea::new([false, true]).auto_shrink([false, true]).max_height(200.0).show(ui, |ui| {
            for idx in 0..32 {
                let response = if self.dragging_bg_source != None && self.dragging_bg_source != Some(self.dragging_bg) && self.dragging_bg_source == Some(idx) {
                    ui.label("")
                } else {
                    if self.dragging_bg_source != None && self.dragging_bg_source != Some(self.dragging_bg) && self.dragging_bg == idx {
                        ui.label("Move Here");
                    }
                    ui.selectable_label(editor_data.current_background == idx, format!("Background Layer {}", idx))
                };

                let id = egui::Id::new("bglayer_item").with(idx);
                if ui.allocate_rect(response.rect, egui::Sense::hover()).hovered() {
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
            if !ui.memory().is_being_dragged(egui::Id::new("bglayer_item").with(source_idx)) {
                level_data.background_layers.insert(self.dragging_bg, level_data.background_layers[source_idx].clone());
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
        egui::Grid::new("background_editor_grid").striped(true).num_columns(2).show(ui, |ui| {
            ui.label("Origin: ").on_hover_text("Origin of the first background layer");
            ui.horizontal(|ui| {
                ui.add(egui::DragValue::new(&mut layer.origin.0).speed(0.75));
                ui.add(egui::DragValue::new(&mut layer.origin.1).speed(0.75));
            });
            ui.end_row();
            ui.label("Scroll Speed: ").on_hover_text("Bigger values make the layer scroll more slowly");
            ui.horizontal(|ui| {
                ui.add(egui::DragValue::new(&mut layer.scroll_speed.0).speed(0.75));
                ui.add(egui::DragValue::new(&mut layer.scroll_speed.1).speed(0.75));
            });
            ui.end_row();
            ui.label("Speed: ").on_hover_text("How many pixels the background scrolls per frame (30 fps)");
            ui.horizontal(|ui| {
                if ui.add(egui::DragValue::new(&mut layer.speed.0).speed(0.75)).changed() {
                    world_panel.background_pos[editor_data.current_background].x = 0.0;
                }
                if ui.add(egui::DragValue::new(&mut layer.speed.1).speed(0.75)).changed() {
                    world_panel.background_pos[editor_data.current_background].y = 0.0;
                }
            });
            ui.end_row();
            ui.label("Spacing: ").on_hover_text("How many pixels of spacing are between each background image");
            ui.horizontal(|ui| {
                ui.add(egui::DragValue::new(&mut layer.spacing.0).speed(1.0));
                ui.add(egui::DragValue::new(&mut layer.spacing.1).speed(1.0));
            });
            ui.end_row();
            ui.label("Transparency: ").on_hover_text("0 (fully opaque) to 7 (fully transparent)");
            ui.add(egui::DragValue::new(&mut layer.transparency).speed(1.0).clamp_range(0..=7));
            ui.end_row();
            ui.label("Repeat: ");
            if ui.selectable_label(layer.repeat_up, "Upwards").clicked() {
                layer.repeat_up = !layer.repeat_up;
            }
            ui.end_row();
            ui.label("Repeat: ");
            if ui.selectable_label(layer.repeat_down, "Downwards").clicked() {
                layer.repeat_down = !layer.repeat_down;
            }
            ui.end_row();
            ui.label("Repeat: ");
            if ui.selectable_label(layer.repeat_horizontally, "Horizontally").clicked() {
                layer.repeat_horizontally = !layer.repeat_horizontally;
            }
            ui.end_row();
            ui.label("Infront of screen: ");
            if ui.selectable_label(layer.in_foreground, "In Foreground").clicked() {
                layer.in_foreground = !layer.in_foreground;
            }
            ui.end_row();
            ui.label("Image/Color: ");
            egui::ComboBox::new("background_image_chooser", "")
                .selected_text(layer.image.to_string())
                .show_ui(ui, |ui| {
                ui.selectable_value(&mut layer.image, BackgroundImage::Color(0), "Whole Color");
                ui.selectable_value(&mut layer.image, BackgroundImage::Bitmap(cnmo_parse::Rect { x: 0, y: 0, w: 0, h: 0 }), "Image");
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
                },
                BackgroundImage::Bitmap(rect) => {
                    ui.label("Rect X: ").on_hover_text("Start of the background image in GFX.BMP");
                    ui.horizontal(|ui| {
                        ui.add(egui::DragValue::new(&mut rect.x).speed(1.0));
                    });
                    ui.end_row();
                    ui.label("Rect Y: ").on_hover_text("Start of the background image in GFX.BMP");
                    ui.horizontal(|ui| {
                        ui.add(egui::DragValue::new(&mut rect.y).speed(1.0));
                    });
                    ui.end_row();
                    ui.label("Rect W: ").on_hover_text("Start of the background image in GFX.BMP");
                    ui.horizontal(|ui| {
                        ui.add(egui::DragValue::new(&mut rect.w).speed(1.0));
                    });
                    ui.end_row();
                    ui.label("Rect H: ").on_hover_text("Start of the background image in GFX.BMP");
                    ui.horizontal(|ui| {
                        ui.add(egui::DragValue::new(&mut rect.h).speed(1.0));
                    });
                    ui.end_row();
                    if ui.button("Pick Image").clicked() {
                        editor_data.selecting_background_color = false;
                        editor_data.selecting_background_image = true;
                    }
                },
            }
            ui.end_row();
        });
    }
    
    fn show_tile_panel(&mut self, editor_data: &mut EditorData, level_data: &mut level_data::LevelData, ui: &mut egui::Ui) {
        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
            ui.heading("Tiles");
        });
        ui.separator();
        if ui.button("New Tile").clicked() {
            level_data.tile_properties.push(level_data::cnmb_types::TileProperties::default());
            editor_data.selected_tiles = vec![level_data.tile_properties.len() - 1];
        }
        self.tile_viewer.edit_tiles = true;
        self.tile_viewer.max_height = None;
        self.tile_viewer.show(ui, level_data, editor_data);
    }
}
