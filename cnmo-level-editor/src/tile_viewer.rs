use eframe::egui;
use cnmo_parse::lparse::level_data;
use crate::instanced_sprites;

pub struct TileViewer {
    max_width: Option<f32>,
    min_width: Option<f32>,
}

impl TileViewer {
    pub fn new() -> Self {
        Self {
            max_width: None,
            min_width: None,
        }
    }

    #[allow(unused)]
    pub fn max_width(self, width: f32) -> Self {
        Self {
            max_width: Some(width),
            min_width: self.min_width,
        }
    }

    pub fn min_width(self, width: f32) -> Self {
        Self {
            min_width: Some(width),
            max_width: self.max_width,
        }
    }

    pub fn show(self, ui: &mut egui::Ui, level_data: &level_data::LevelData) {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Min).with_main_wrap(true), |ui| {
            let mut main_rect = ui.available_rect_before_wrap();
            main_rect.set_width(main_rect.width().clamp(self.min_width.unwrap_or(0.0), self.max_width.unwrap_or(f32::INFINITY)));
            let icon_size = 32.0;
            let padding = 2.0;
            let icons_per_row = (main_rect.width() / (icon_size + padding)) as usize;

            egui::ScrollArea::vertical()
                .auto_shrink([false, true])
                .show_rows(ui,
                    icon_size + padding,
                    level_data.tile_properties.len() / icons_per_row,
                    |ui, _row_range| {
                instanced_sprites::InstancedSprites::new()
                    .paint(ui, main_rect);
                
                // for _tile in level_data.tile_properties.iter() {
                //     test_triangle.custom_painting(ui, (
                //         tile.frames[0].0 as u32,
                //         tile.frames[0].1 as u32,
                //         32, 32,
                //     ));
                // }
            });
        });
    }
}
