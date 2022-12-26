use eframe::egui;
use cnmo_parse::lparse::level_data;

pub fn show_metadata_panel(editor_mode: &mut super::EditorMode, level_data: &mut level_data::LevelData, ui: &mut egui::Ui) {
    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
        ui.heading("File");
    });
    let _ = ui.button("New Level");
    let _ = ui.button("Load Level");
    let _ = ui.button("Save Level");
    let _ = ui.button("Decompile Level");
    let _ = ui.button("Compile Level");
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

pub fn show_properties_panel(editor_mode: &mut super::EditorMode, level_data: &mut level_data::LevelData, ui: &mut egui::Ui) {
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

fn show_tile_panel(_level_data: &mut level_data::LevelData, ui: &mut egui::Ui) {
    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
        ui.heading("Tiles");
    });
}
