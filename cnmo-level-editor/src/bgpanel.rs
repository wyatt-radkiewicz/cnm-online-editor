use crate::editor_data::EditorData;
use crate::instanced_sprites::{InstancedSprites, Sprite};
use cnmo_parse::lparse::level_data::{cnmb_types::BackgroundImage, LevelData};
use cnmo_parse::Rect;
use eframe::egui;
//use crate::common_gfx::GfxCommonResources;
use crate::camera::Camera;

crate::create_instance_resource!(BgPanelSpriteInstances);

pub struct BgPanel {
    pub picker_origin: (i32, i32),
    pub min_bound: (i32, i32),
    pub max_bound: (i32, i32),
    pub grid_size: i32,
    scroll_offset: f32,
}

impl BgPanel {
    pub fn new() -> Self {
        Self {
            picker_origin: (0, 0),
            min_bound: (0, 0),
            max_bound: (0, 0),
            grid_size: 8,
            scroll_offset: 0.0,
        }
    }

    fn update_picking(&mut self, pos: (i32, i32), is_press: bool) {
        if is_press {
            self.picker_origin = pos;
            self.min_bound = pos;
            self.max_bound = pos;
        } else {
            self.min_bound = (
                pos.0.min(self.picker_origin.0),
                pos.1.min(self.picker_origin.1),
            );
            self.max_bound = (
                pos.0.max(self.picker_origin.0),
                pos.1.max(self.picker_origin.1),
            );
        }
    }

    pub fn update(
        &mut self,
        ui: &mut egui::Ui,
        level_data: &mut LevelData,
        editor_data: &mut EditorData,
    ) {
        if editor_data.selecting_background_image {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let (rect, response) = ui.allocate_exact_size(
                        ui.available_size(),
                        //egui::vec2(
                        //    scale,
                        //    (editor_data.gfx_size.1 as f32 / editor_data.gfx_size.0 as f32) * scale,
                        //),
                        egui::Sense::hover().union(egui::Sense::click_and_drag()),
                    );
                    let cw = editor_data.gfx_size.0 as f32;
                    let ch = (rect.height() / rect.width()) * editor_data.gfx_size.0 as f32;
                    self.scroll_offset -= response.ctx.input().scroll_delta.y;
                    self.scroll_offset = self
                        .scroll_offset
                        .clamp(0.0, editor_data.gfx_size.1 as f32 - ch);
                    let pointer_pos = {
                        let pos = response
                            .interact_pointer_pos()
                            .unwrap_or(egui::pos2(0.0, 0.0))
                            - rect.min.to_vec2();

                        egui::vec2(
                            pos.x / rect.width() * cw,
                            pos.y / rect.height() * ch + self.scroll_offset,
                        )
                    };

                    if response.dragged() {
                        self.update_picking(
                            (
                                (pointer_pos.x as i32 / self.grid_size * self.grid_size)
                                    .clamp(0, editor_data.gfx_size.0 as i32),
                                (pointer_pos.y as i32 / self.grid_size * self.grid_size)
                                    .clamp(0, editor_data.gfx_size.1 as i32),
                            ),
                            response.drag_started(),
                        );
                    }

                    if response.drag_released() {
                        editor_data.selecting_background_color = false;
                        editor_data.selecting_background_image = false;
                        level_data.background_layers[editor_data.current_background].image =
                            BackgroundImage::Bitmap(Rect {
                                x: self.min_bound.0,
                                y: self.min_bound.1,
                                w: self.max_bound.0 - self.min_bound.0,
                                h: self.max_bound.1 - self.min_bound.1,
                            });
                    }

                    let mut sprites = vec![];
                    let camera = Camera::new().with_projection(cw, ch, None, true);

                    sprites.push(Sprite::new(
                        (0.0, 0.0, 0.0),
                        (editor_data.gfx_size.0 as f32, editor_data.gfx_size.1 as f32),
                        (
                            0.0,
                            self.scroll_offset,
                            editor_data.gfx_size.0 as f32,
                            editor_data.gfx_size.1 as f32,
                        ),
                    ));
                    if response.dragged() {
                        sprites.push(Sprite::new_pure_color(
                            (
                                self.min_bound.0 as f32,
                                self.min_bound.1 as f32 - self.scroll_offset,
                                0.0,
                            ),
                            (
                                self.max_bound.0 as f32 - self.min_bound.0 as f32,
                                self.max_bound.1 as f32 - self.min_bound.1 as f32,
                            ),
                            (1.0, 1.0, 0.0, 0.5),
                        ));
                    }

                    InstancedSprites::new()
                        .with_camera(camera)
                        .with_sprites(sprites)
                        .paint::<BgPanelSpriteInstances>(ui, rect);
                });
        } else {
            let min_size = ui.available_size().x.min(ui.available_size().y);
            let (rect, response) =
                ui.allocate_exact_size(egui::Vec2::splat(min_size), egui::Sense::click_and_drag());
            let camera = Camera::new().with_projection(16.0, 16.0, None, true);
            let pointer_pos = response
                .interact_pointer_pos()
                .unwrap_or(egui::pos2(0.0, 0.0))
                - rect.min.to_vec2();

            if response.clicked() {
                self.picker_origin = (
                    (pointer_pos.x / min_size * 16.0) as i32,
                    (pointer_pos.y / min_size * 16.0) as i32,
                );
                if self.picker_origin.0 >= 0
                    && self.picker_origin.0 < 16
                    && self.picker_origin.1 >= 0
                    && self.picker_origin.1 < 16
                {
                    editor_data.selecting_background_color = false;
                    editor_data.selecting_background_image = false;
                    level_data.background_layers[editor_data.current_background].image =
                        BackgroundImage::Color(
                            (self.picker_origin.1 * 16 + self.picker_origin.0) as u8,
                        );
                }
            }

            let mut sprites = vec![];

            // Draw the palette
            for y in 0..16 {
                for x in 0..16 {
                    let color = editor_data.palette[y * 16 + x];
                    sprites.push(Sprite::new_pure_color(
                        (x as f32, y as f32, 0.0),
                        (1.0, 1.0),
                        (
                            color[0] as f32 / 255.0,
                            color[1] as f32 / 255.0,
                            color[2] as f32 / 255.0,
                            1.0,
                        ),
                    ));
                }
            }

            InstancedSprites::new()
                .with_camera(camera)
                .with_sprites(sprites)
                .paint::<BgPanelSpriteInstances>(ui, rect);
        }
    }
}
