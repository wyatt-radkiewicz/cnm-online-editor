use cnmo_parse::lparse::level_data::cnmb_types::Cells;
use cnmo_parse::lparse::level_data::cnmb_types::TileId;
use eframe::egui;
use cnmo_parse::lparse::level_data;
use crate::instanced_sprites;
use crate::camera;
use crate::editor_data::EditorData;

crate::create_instance_resource!(TileViewerSpriteInstances);
crate::create_instance_resource!(TileViewerDraggingSpriteInstances);

pub struct TileViewer {
    pub max_width: Option<f32>,
    pub min_width: Option<f32>,
    pub max_height: Option<f32>,
    pub edit_tiles: bool,
    drag_pos: Option<egui::Vec2>,
    drag_source: u16,
    drag_dest: u16,
}

impl TileViewer {
    pub fn new(max_width: Option<f32>, min_width: Option<f32>, max_height: Option<f32>, edit_tiles: bool) -> Self {
        Self {
            max_width,
            min_width,
            max_height,
            edit_tiles,
            drag_pos: None,
            drag_source: 0,
            drag_dest: 0,
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, level_data: &mut level_data::LevelData, editor_data: &mut EditorData) {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Min).with_main_wrap(true), |ui| {
            let icon_size = 32.0;
            let padding = 2.0;

            egui::ScrollArea::vertical()
                .auto_shrink([false, true])
                .max_height(self.max_height.unwrap_or(f32::INFINITY))
                .show(ui, |ui| {
                let mut main_rect = ui.available_rect_before_wrap();
                main_rect.set_width(main_rect.width().clamp(self.min_width.unwrap_or(0.0), self.max_width.unwrap_or(f32::INFINITY)));
                let icons_per_row = ((main_rect.width() - padding) / (icon_size + padding)) as usize;
                main_rect.set_height((level_data.tile_properties.len() / icons_per_row + 1) as f32 * (icon_size + padding) + padding);
                let response = ui.allocate_rect(main_rect, egui::Sense::click_and_drag());

                let mut camera = camera::Camera::new();
                camera.set_projection(main_rect.width(), main_rect.height(), None, true);
                let mut sprites = vec![];
                let (mut x, mut y) = (padding, padding);
                let (mut ix, mut iy) = (0, 0);
                let mut selected_tiles_poses = Vec::new();
                let add_padding = |x: &mut f32, y: &mut f32, ix: &mut i32, iy: &mut i32| {
                    *x += icon_size + padding;
                    *ix += 1;
                    if *x + icon_size + padding + 5.0 > main_rect.width() {
                        *x = padding;
                        *ix = 0;
                        *y += icon_size + padding;
                        *iy += 1;
                    }
                };
                let mut dragged_outside = true;
                if self.drag_pos != None && self.drag_dest == 0 && self.drag_source != 0 {
                    dragged_outside = false;
                    add_padding(&mut x, &mut y, &mut ix, &mut iy);
                }
                for (idx, tile) in level_data.tile_properties.iter().enumerate() {
                    let (mut fx, mut fy) = (0.0, 0.0);
                    if let Some((x, y)) = tile.frames.get(0) {
                        fx = *x as f32 * 32.0;
                        fy = *y as f32 * 32.0;
                    }
                    if response.clicked_by(egui::PointerButton::Primary) {
                        let pos = response.interact_pointer_pos().unwrap() - main_rect.min;
                        if pos.x >= x && pos.x <= x + icon_size + padding && pos.y >= y && pos.y <= y + icon_size + padding {
                            if response.ctx.input().modifiers.ctrl {
                                if !editor_data.selected_tiles.contains(&idx) {
                                    editor_data.selected_tiles.push(idx);
                                }
                            } else {
                                editor_data.selected_tiles = vec![idx];
                            }
                            dragged_outside = false;
                        }
                    }
                    if response.drag_started() && !response.ctx.input().modifiers.ctrl && response.hovered() {
                        let pos = response.interact_pointer_pos().unwrap() - main_rect.min;
                        if pos.x >= x && pos.x <= x + icon_size + padding && pos.y >= y && pos.y <= y + icon_size + padding {
                            self.drag_pos = Some(pos);
                            self.drag_source = idx as u16;
                            self.drag_dest = idx as u16;
                            dragged_outside = false;
                        }
                    }
                    if self.drag_pos != None {
                        let pos = response.interact_pointer_pos().unwrap_or_default() - main_rect.min;
                        if pos.x >= x && pos.x <= x + icon_size + padding && pos.y >= y && pos.y <= y + icon_size + padding {
                            self.drag_dest = idx as u16;
                            dragged_outside = false;
                            if self.drag_dest != self.drag_source && self.drag_dest != 0 {
                                add_padding(&mut x, &mut y, &mut ix, &mut iy);
                            }
                        }
                    }
                    if editor_data.selected_tiles.contains(&idx) {
                        selected_tiles_poses.push((ix, iy, idx));
                    }
                    sprites.push(instanced_sprites::Sprite::new(
                        (x, y, 0.0),
                        (icon_size, icon_size),
                        (fx, fy, 32.0, 32.0),
                    ));
                    if editor_data.selected_tiles.contains(&idx) {
                        sprites.push(instanced_sprites::Sprite::new_pure_color(
                            (x - padding / 2.0, y - padding / 2.0, 0.0),
                            (icon_size + padding, icon_size + padding),
                            (0.0, 1.0, 1.0, 0.2),
                        ));
                    }
                    add_padding(&mut x, &mut y, &mut ix, &mut iy);
                }
                if response.ctx.input().pointer.primary_released() && dragged_outside && self.drag_pos == None && response.hovered() {
                    editor_data.selected_tiles = vec![];
                    editor_data.viewer_selection = None;
                }
                if response.dragged() && self.drag_pos != None {
                    let pos = response.interact_pointer_pos().unwrap() - main_rect.min;
                    self.drag_pos = Some(pos);
                }
                if response.drag_released() && self.drag_pos != None {
                    editor_data.cells_history = vec![];

                    // Complete the drag
                    let change_refrences = |old_id: u16, new_id: u16, cells: &mut Cells| {
                        for cell in cells.cells_mut().iter_mut() {
                            if cell.foreground.0 == Some(old_id) {
                                cell.foreground.0 = Some(new_id);
                            } else {
                                if let Some(foreground) = &mut cell.foreground.0 {
                                    if *foreground > old_id {
                                        *foreground -= 1;
                                    }
                                    if *foreground >= new_id {
                                        *foreground += 1;
                                    }
                                }
                            }
                            if cell.background.0 == Some(old_id) {
                                cell.background.0 = Some(new_id);
                            } else {
                                if let Some(background) = &mut cell.background.0 {
                                    if *background > old_id {
                                        *background -= 1;
                                    }
                                    if *background >= new_id {
                                        *background += 1;
                                    }
                                }
                            }
                        }
                    };

                    self.drag_pos = None;
                    let src = level_data.tile_properties[self.drag_source as usize].clone();
                    if self.drag_dest != self.drag_source {
                        if dragged_outside {
                            level_data.tile_properties.push(src);
                            level_data.tile_properties.remove(self.drag_source as usize);
                            change_refrences(self.drag_source, (level_data.tile_properties.len() - 1) as u16, &mut level_data.cells);
                            editor_data.selected_tiles = vec![level_data.tile_properties.len() - 1];
                        } else {
                            level_data.tile_properties.insert(self.drag_dest as usize, src);
                            if self.drag_dest < self.drag_source {
                                change_refrences(self.drag_source, self.drag_dest, &mut level_data.cells);
                                level_data.tile_properties.remove(self.drag_source as usize + 1);
                                editor_data.selected_tiles = vec![self.drag_dest as usize];
                            } else {
                                change_refrences(self.drag_source, self.drag_dest - 1, &mut level_data.cells);
                                level_data.tile_properties.remove(self.drag_source as usize);
                                editor_data.selected_tiles = vec![self.drag_dest as usize - 1];
                            }
                        }
                    }
                }

                if let Some(drag_pos) = self.drag_pos {
                    if self.drag_source != self.drag_dest {
                        let drag_src = &level_data.tile_properties[self.drag_source as usize];
                        let mut sprite = instanced_sprites::Sprite::new(
                            (drag_pos.x, drag_pos.y, 0.0),
                            (icon_size, icon_size),
                            (drag_src.frames[0].0 as f32 * 32.0, drag_src.frames[0].1 as f32 * 32.0, 32.0, 32.0),
                        );
                        sprite.tint = [0.8, 0.85, 0.95, 0.9];
                        sprites.push(sprite);
                    }
                }

                if selected_tiles_poses.len() > 0 && editor_data.selected_tiles.len() > 0 {
                    editor_data.has_copied_tiles = false;
                    let max_x = selected_tiles_poses.iter().max_by(|a, b| (a.0).cmp(&b.0)).unwrap().0;
                    let max_y = selected_tiles_poses.iter().max_by(|a, b| (a.1).cmp(&b.1)).unwrap().1;
                    let min_x = selected_tiles_poses.iter().min_by(|a, b| (a.0).cmp(&b.0)).unwrap().0;
                    let min_y = selected_tiles_poses.iter().min_by(|a, b| (a.1).cmp(&b.1)).unwrap().1;
                    let mut cells = Cells::new((max_x - min_x + 1) as usize, (max_y - min_y + 1) as usize);
                    for selected in selected_tiles_poses {
                        cells.get_cell_mut(selected.0 - min_x, selected.1 - min_y).foreground = TileId(Some(selected.2 as u16));
                        cells.get_cell_mut(selected.0 - min_x, selected.1 - min_y).background = TileId(Some(selected.2 as u16));
                    }
                    editor_data.viewer_selection = Some(cells);
                }

                instanced_sprites::InstancedSprites::new()
                    .with_camera(camera)
                    .with_sprites(sprites)
                    .paint::<TileViewerSpriteInstances>(ui, main_rect);
            });
        });
    }
}
