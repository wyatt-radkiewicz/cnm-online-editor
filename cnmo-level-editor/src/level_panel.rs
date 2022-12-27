use eframe::egui;
use cnmo_parse::lparse::level_data;

pub fn show_metadata_panel(editor_mode: &mut super::EditorMode, level_data: &mut level_data::LevelData, ui: &mut egui::Ui) {
    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
        ui.heading("File");
    });
    if ui.button("New Level").clicked() {
        *level_data = level_data::LevelData::from_version(1).expect("Can't create version 1 level type?");
        log::info!("Created a new level!");
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
            let cnmb = cnmo_parse::lparse::LParse::new(1);
            let cnms = cnmo_parse::lparse::LParse::new(1);
            if let (Ok(mut cnmb), Ok(mut cnms)) = (cnmb, cnms) {
                level_data.save(&mut cnmb, &mut cnms);
                match (cnmb.save_to_file(cnmb_path), cnmb.save_to_file(cnms_path)) {
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
            ui.selectable_value(editor_mode, super::EditorMode::Level, "Level");
            ui.selectable_value(editor_mode, super::EditorMode::Background, "Background");
            ui.selectable_value(editor_mode, super::EditorMode::Tile, "Tile");
        });
        ui.end_row();
    });
    ui.separator();
}

pub fn show_propeties_panel(editor_mode: &mut super::EditorMode, level_data: &mut level_data::LevelData, ui: &mut egui::Ui) {
    match editor_mode {
        &mut super::EditorMode::Level => show_level_panel(level_data, ui),
        &mut super::EditorMode::Background => show_background_panel(level_data, ui),
        &mut super::EditorMode::Tile => show_tile_panel(level_data, ui),
    }
}

fn show_level_panel(_level_data: &mut level_data::LevelData, ui: &mut egui::Ui) {
    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
        ui.heading("Tiles");
    });
    ui.separator();
    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
        ui.heading("Spawners");
    });
}

fn show_background_panel(_level_data: &mut level_data::LevelData, ui: &mut egui::Ui) {
    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
        ui.heading("Background Properties");
    });
}

fn show_tile_panel(level_data: &mut level_data::LevelData, ui: &mut egui::Ui) {
    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
        ui.heading("Tiles");
    });
    ui.separator();
    use crate::tile_viewer::TileViewer;
    TileViewer::new()
        .min_width(50.0)
        .show(ui, level_data);
}
