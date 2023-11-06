use std::collections::VecDeque;

use crate::editor_data::{EditorData, Tool};
use crate::instanced_sprites::{InstancedSprites, Sprite};
use cnmo_parse::lparse::level_data;
use cnmo_parse::lparse::level_data::cnmb_types::{BackgroundLayer, Cells, TileId, TileProperties};
use cnmo_parse::lparse::level_data::cnms_types::wobj_type::WobjType;
use cnmo_parse::lparse::level_data::cnms_types::SpawningCriteria;
use eframe::egui;
use level_data::cnmb_types::BackgroundImage;
//use crate::common_gfx::GfxCommonResources;
use crate::camera::Camera;

crate::create_instance_resource!(WorldPanelSpriteInstances);

pub struct WorldPanel {
    pub camera: Camera,
    pub background_pos: Vec<cgmath::Vector2<f32>>,
    pub gray_other: bool,
    pub background_range: std::ops::RangeInclusive<usize>,
    pub resizing_bounds: (bool, bool, bool, bool),
    pub grabbing_resize: bool,
    pub show_grid: bool,
    pub brush_origin: (i32, i32),
    pub copy_selection: Option<(i32, i32, i32, i32)>,
    pub editing_background: bool,
    pub show_original_screen_size: bool,
    panning_speed: f32,
    pub right_clicked_spawner_idx: Option<usize>,
    pub close_context_menu: bool,
    pub hovered_on_context_menu: bool,
}

impl WorldPanel {
    pub fn new() -> Self {
        Self {
            camera: Camera::new(),
            background_pos: (0..64).map(|_| cgmath::vec2(0.0, 0.0)).collect(),
            gray_other: true,
            background_range: 0..=31,
            resizing_bounds: (false, false, false, false),
            grabbing_resize: false,
            show_grid: true,
            brush_origin: (0, 0),
            copy_selection: None,
            editing_background: false,
            show_original_screen_size: false,
            panning_speed: 0.0,
            right_clicked_spawner_idx: None,
            close_context_menu: false,
            hovered_on_context_menu: false,
        }
    }

    pub fn update(
        &mut self,
        ui: &mut egui::Ui,
        level_data: &mut level_data::LevelData,
        editor_data: &mut EditorData,
    ) {
        // Draw the level
        let (rect, response) = ui.allocate_exact_size(
            ui.available_size(),
            egui::Sense::hover().union(egui::Sense::click_and_drag()),
        );
        if response.clicked() {
            editor_data.editing_text = None;
        }
        self.update_camera(ui, &rect, &response, editor_data);
        let pointer_pos = {
            let cam_size = self.camera.get_proj_size_world_space();
            let top_left = self.camera.get_top_left_world_space();
            let pt = ui.ctx().pointer_latest_pos().unwrap_or(rect.min);
            egui::pos2(
                (pt.x - rect.min.x) / rect.width() * cam_size.x + top_left.x,
                (pt.y - rect.min.y) / rect.height() * cam_size.y + top_left.y,
            )
        };
        let mut sprites = vec![];
        self.update_level_background(
            &mut sprites,
            ui,
            level_data,
            editor_data,
            false,
            &rect,
            &response,
            &pointer_pos,
        );
        if !self.editing_background {
            self.update_level_tiles(
                &mut sprites,
                ui,
                level_data,
                &rect,
                &response,
                editor_data,
                &pointer_pos,
            );
            self.update_level_spawners(
                &mut sprites,
                ui,
                level_data,
                &rect,
                &response,
                editor_data,
                &pointer_pos,
            );
        }
        self.update_level_background(
            &mut sprites,
            ui,
            level_data,
            editor_data,
            true,
            &rect,
            &response,
            &pointer_pos,
        );
        if !self.editing_background {
            self.show_grid(
                &mut sprites,
                ui,
                level_data,
                editor_data,
                &rect,
                &response,
                &pointer_pos,
            );
        }
        if response.hovered()
            && (response.ctx.input().modifiers.ctrl || response.ctx.input().modifiers.mac_cmd)
            && response.ctx.input().key_pressed(egui::Key::Z)
        {
            if let Some(history) = editor_data.cells_history.pop() {
                level_data.cells = history.0;
                level_data.spawners = history.1;
                editor_data.selected_spawner = None;
                log::info!("Undid transformation");
            } else {
                log::info!("There are no transformations in the history buffer!");
            }
        }
        let grid_size = if matches!(editor_data.tool, Tool::Spawners) {
            editor_data.spawner_grid_size
        } else {
            32.0
        };
        editor_data.info_bar = format!(
            "pixel: ({}, {}), snapped: ({}, {}), grid: ({}, {})",
            pointer_pos.x as i32,
            pointer_pos.y as i32,
            (pointer_pos.x / grid_size).floor() as i32,
            (pointer_pos.y / grid_size).floor() as i32,
            ((pointer_pos.x / grid_size).floor() * grid_size) as i32,
            ((pointer_pos.y / grid_size).floor() * grid_size) as i32,
        );
        InstancedSprites::new()
            .with_camera(self.camera.clone())
            .with_sprites(sprites)
            .paint::<WorldPanelSpriteInstances>(ui, rect);
    }

    fn update_camera(
        &mut self,
        ui: &mut egui::Ui,
        rect: &egui::Rect,
        response: &egui::Response,
        editor_data: &EditorData,
    ) {
        self.camera
            .set_projection(rect.width(), rect.height(), Some(32.0 * 10.0), false);
        if response.dragged() && response.dragged_by(egui::PointerButton::Middle) {
            ui.output().cursor_icon = egui::CursorIcon::Grab;
            let size = self.camera.get_proj_size_world_space();
            let scaled_delta = cgmath::vec2(
                ((response.drag_delta().x * response.ctx.pixels_per_point()) / rect.width())
                    * size.x,
                ((response.drag_delta().y * response.ctx.pixels_per_point()) / rect.height())
                    * size.y,
            );
            self.camera.pos -= scaled_delta;
        }
        if response.hovered() {
            let cam_speed = self.panning_speed * editor_data.dt.as_secs_f32();
            let mut used_input = false;
            if response.ctx.input().key_down(egui::Key::ArrowRight) {
                self.camera.pos.x += cam_speed;
                used_input = true;
            }
            if response.ctx.input().key_down(egui::Key::ArrowLeft) {
                self.camera.pos.x -= cam_speed;
                used_input = true;
            }
            if response.ctx.input().key_down(egui::Key::ArrowDown) {
                self.camera.pos.y += cam_speed;
                used_input = true;
            }
            if response.ctx.input().key_down(egui::Key::ArrowUp) {
                self.camera.pos.y -= cam_speed;
                used_input = true;
            }
            if used_input {
                self.panning_speed += 32.0 * 8.0 * editor_data.dt.as_secs_f32();
                self.panning_speed = self.panning_speed.min(32.0 * 128.0);
            } else {
                self.panning_speed = 32.0 * 8.0;
            }

            self.camera.zoom -= response.ctx.input().scroll_delta.y / 1000.0;
            self.camera.zoom = self.camera.zoom.clamp(0.02, 1.8);
        } else {
            self.panning_speed = 32.0 * 8.0;
        }
    }

    fn get_background_pos(
        &self,
        layer: &BackgroundLayer,
        idx: usize,
        scale: f32,
    ) -> cgmath::Vector2<f32> {
        let pos = cgmath::vec2(self.background_pos[idx].x, self.background_pos[idx].y);
        let cx = if layer.scroll_speed.0 < f32::EPSILON {
            0.0
        } else {
            self.camera.pos.x / layer.scroll_speed.0
        };
        let cy = if layer.scroll_speed.1 < f32::EPSILON {
            0.0
        } else {
            self.camera.pos.y / layer.scroll_speed.1
        };
        cgmath::vec2(
            (layer.origin.0 + pos.x) * scale,
            (layer.origin.1 + pos.y) * scale,
        ) + cgmath::vec2(self.camera.pos.x - cx, self.camera.pos.y - cy)
    }

    fn update_level_background(
        &mut self,
        sprites: &mut Vec<Sprite>,
        _ui: &mut egui::Ui,
        level_data: &mut level_data::LevelData,
        editor_data: &mut EditorData,
        foreground: bool,
        main_rect: &egui::Rect,
        response: &egui::Response,
        pointer_pos: &egui::Pos2,
    ) {
        let top_left = self.camera.get_top_left_world_space();
        let cam_size = self.camera.get_proj_size_world_space();
        let scale = cam_size.y / 240.0 as f32;
        let mut selected_layer = None;

        for idx in self.background_range.clone() {
            let layer = match level_data.background_layers.get_mut(idx) {
                Some(layer) => layer,
                None => continue,
            };
            if layer.in_foreground != foreground {
                continue;
            }
            if let BackgroundImage::Color(color_index) = layer.image {
                let pal = &editor_data.palette[color_index as usize];
                sprites.push(Sprite::new_pure_color(
                    (top_left.x, top_left.y, 0.0),
                    (cam_size.x, cam_size.y),
                    (
                        pal[0] as f32 / 255.0,
                        pal[1] as f32 / 255.0,
                        pal[2] as f32 / 255.0,
                        1.0,
                    ),
                ));
                continue;
            }
            if let BackgroundImage::Bitmap(rect) = layer.image {
                let padded_size = cgmath::vec2(
                    rect.w as f32 + layer.spacing.0 as f32,
                    rect.h as f32 + layer.spacing.1 as f32,
                ) * scale;
                if padded_size.x < f32::EPSILON || padded_size.y < f32::EPSILON {
                    continue;
                }

                if self.editing_background && editor_data.current_background == idx {
                    if response.dragged_by(egui::PointerButton::Primary) {
                        let delta = response.drag_delta();
                        layer.origin.0 += (delta.x / main_rect.width()) * cam_size.x / scale;
                        layer.origin.1 += (delta.y / main_rect.height()) * cam_size.y / scale;
                    }
                    if response.drag_released() {
                        layer.origin.0 = (layer.origin.0 / 4.0).round() * 4.0;
                        layer.origin.1 = (layer.origin.1 / 4.0).round() * 4.0;
                    }
                }

                self.background_pos[idx].x += layer.speed.0 * 30.0 * editor_data.dt.as_secs_f32();
                self.background_pos[idx].y += layer.speed.1 * 30.0 * editor_data.dt.as_secs_f32();
                let new_pos = self.get_background_pos(layer, idx, scale);

                let start_x = if layer.repeat_horizontally {
                    let new_pos = new_pos.x.rem_euclid(padded_size.x) - padded_size.x;
                    new_pos + ((top_left.x - new_pos) / padded_size.x).floor() * padded_size.x
                } else {
                    new_pos.x
                };
                let mut y = if layer.repeat_up || (layer.repeat_down && top_left.y > new_pos.y) {
                    let new_pos = new_pos.y.rem_euclid(padded_size.y) - padded_size.y;
                    new_pos + ((top_left.y - new_pos) / padded_size.y).floor() * padded_size.y
                } else {
                    new_pos.y
                };
                let cap_width = layer.scroll_speed.0 < f32::EPSILON
                    && rect.w >= 320
                    && !layer.repeat_horizontally;
                let width = if cap_width {
                    cam_size.x
                } else {
                    rect.w as f32 * scale
                };
                let src_width = if cap_width { 320 } else { rect.w };
                for _ in 0..(if layer.repeat_up || layer.repeat_down {
                    (cam_size.y / padded_size.y).ceil() as i32 * 2 + 1
                } else {
                    1
                }) {
                    let mut x = start_x;
                    if y - f32::EPSILON * 2.0 > new_pos.y + padded_size.y / 2.0
                        && layer.repeat_up
                        && !layer.repeat_down
                    {
                        break;
                    }
                    for _ in 0..(if layer.repeat_horizontally {
                        (cam_size.x / padded_size.x).ceil() as i32 * 2 + 1
                    } else {
                        1
                    }) {
                        let real_pos = cgmath::vec2(x - cam_size.x / 2.0, y - cam_size.y / 2.0);
                        let height = rect.h as f32 * scale;

                        if pointer_pos.x > real_pos.x
                            && pointer_pos.y > real_pos.y
                            && pointer_pos.x < real_pos.x + width
                            && pointer_pos.y < real_pos.y + height
                            && response.clicked()
                        {
                            if cap_width {
                                selected_layer = Some(idx);
                            } else {
                                let local_pos = (
                                    (((pointer_pos.x - real_pos.x) / scale) as i32)
                                        .clamp(0, rect.w - 1),
                                    (((pointer_pos.y - real_pos.y) / scale) as i32)
                                        .clamp(0, rect.h - 1),
                                );
                                if !editor_data.opaques[(local_pos.0 + rect.x) as usize]
                                    [(local_pos.1 + rect.y) as usize]
                                {
                                    selected_layer = Some(idx);
                                }
                            }
                        }

                        let mut sprite = Sprite::new(
                            (real_pos.x, real_pos.y, 0.0),
                            (width, height),
                            (
                                rect.x as f32,
                                rect.y as f32,
                                src_width as f32,
                                rect.h as f32,
                            ),
                        );
                        let tint = if self.editing_background && editor_data.gray_out_background {
                            if editor_data.current_background == idx {
                                (1.0, 1.0, 1.0, 0.0)
                            } else {
                                (0.4, 0.4, 0.4, 0.2)
                            }
                        } else {
                            (1.0, 1.0, 1.0, 0.0)
                        };
                        sprite.tint = [
                            tint.0,
                            tint.1,
                            tint.2,
                            ((1.0 - tint.3)
                                - (layer.transparency as f32
                                    / (level_data::consts::CLEAR as f32 - f32::EPSILON)))
                                .clamp(0.0, 1.0),
                        ];
                        sprites.push(sprite);
                        x += padded_size.x;
                    }
                    y += padded_size.y;
                }
            }
        }

        if let Some(layer) = selected_layer {
            if self.editing_background {
                editor_data.current_background = layer;
                log::info!("Picked Background ID: {}", layer);
            }
        }
    }

    fn update_level_tiles(
        &mut self,
        sprites: &mut Vec<Sprite>,
        _ui: &mut egui::Ui,
        level_data: &mut level_data::LevelData,
        _rect: &egui::Rect,
        response: &egui::Response,
        editor_data: &mut EditorData,
        pointer_pos: &egui::Pos2,
    ) {
        let top_left = self.camera.get_top_left_world_space();
        //let cam_size = self.camera.get_proj_size_world_space();
        let start = cgmath::vec2(
            (top_left.x / 32.0).floor() as i32,
            (top_left.y / 32.0).floor() as i32,
        );
        let end = cgmath::vec2(
            (self.camera.get_bottom_right_world_space().x / 32.0).ceil() as i32 + 1,
            (self.camera.get_bottom_right_world_space().y / 32.0).ceil() as i32 + 1,
        );

        let mx = (pointer_pos.x / 32.0).floor() as i32;
        let my = (pointer_pos.y / 32.0).floor() as i32;
        let shift = response.ctx.input().modifiers.shift;
        let tile_placing_enabled = (response.ctx.input().pointer.primary_down()
            || response.ctx.input().pointer.secondary_down())
            && !self.grabbing_resize;
        if matches!(editor_data.tool, Tool::Brush) {
            if tile_placing_enabled && shift && response.ctx.input().pointer.primary_clicked() {
                self.copy_selection = Some((mx, my, mx, my));
                self.brush_origin = (mx, my);
            }
            if let Some(copy_selection) = self.copy_selection.as_mut() {
                copy_selection.0 = self.brush_origin.0.min(mx);
                copy_selection.1 = self.brush_origin.1.min(my);
                copy_selection.2 = self.brush_origin.0.max(mx);
                copy_selection.3 = self.brush_origin.1.max(my);
                if !response.ctx.input().pointer.primary_down() {
                    let mut viewer_selection = Cells::new(
                        (copy_selection.2 - copy_selection.0 + 1) as usize,
                        (copy_selection.3 - copy_selection.1 + 1) as usize,
                    );
                    level_data.cells.paste(
                        &mut viewer_selection,
                        (copy_selection.0, copy_selection.1),
                        (copy_selection.2, copy_selection.3),
                        (0, 0),
                    );
                    editor_data.viewer_selection = Some(viewer_selection);
                    editor_data.selected_tiles = vec![];
                    self.copy_selection = None;
                    editor_data.has_copied_tiles = true;
                }
            }
        } else {
            self.copy_selection = None;
            editor_data.has_copied_tiles = false;
            editor_data.viewer_selection = None;
        }

        if matches!(editor_data.tool, Tool::Light) {
            let tile_ref = level_data.cells.get_cell_mut(mx, my);
            if response.ctx.input().pointer.primary_down() && response.hovered() {
                tile_ref.light = editor_data.light_tool_level;
            } else if response.ctx.input().pointer.secondary_down() && response.hovered() {
                tile_ref.light = level_data::consts::LIGHT_NORMAL;
            }
        }

        if tile_placing_enabled && self.copy_selection == None {
            if matches!(editor_data.tool, Tool::Eraser)
                && (response.ctx.input().pointer.primary_clicked()
                    || response.ctx.input().pointer.secondary_clicked())
            {
                editor_data
                    .cells_history
                    .push((level_data.cells.clone(), level_data.spawners.clone()));
                if editor_data.cells_history.len() > 512 {
                    editor_data.cells_history.remove(0);
                }
            }
            let tile_ref = level_data.cells.get_cell_mut(mx, my);
            if matches!(editor_data.tool, Tool::Eraser) {
                if response.ctx.input().pointer.primary_down() && response.hovered() {
                    if editor_data.foreground_placing {
                        tile_ref.foreground = level_data::cnmb_types::TileId(None);
                    } else {
                        tile_ref.background = level_data::cnmb_types::TileId(None);
                    }
                }
                if response.ctx.input().pointer.secondary_down() && response.hovered() {
                    if editor_data.foreground_placing {
                        tile_ref.background = level_data::cnmb_types::TileId(None);
                    } else {
                        tile_ref.foreground = level_data::cnmb_types::TileId(None);
                    }
                }
            }
        } else if self.copy_selection == None {
            self.brush_origin = (mx, my);
        }

        for row in start.y..=end.y {
            for column in start.x..=end.x {
                let tile = level_data.cells.get_cell(column, row);
                let pos = (column as f32 * 32.0, row as f32 * 32.0, 0.0);
                let mut drawer = |props: &TileProperties, foreground: bool| {
                    let idx = (editor_data.time_past.as_secs_f32()
                        * (30.0 / props.anim_speed.0 as f32))
                        as usize
                        % props.frames.len();
                    let tint = if self.gray_other
                        && editor_data.foreground_placing != foreground
                        && !matches!(editor_data.tool, Tool::Light) 
                    {
                        0.5
                    } else {
                        1.0
                    };
                    let mut sprite = Sprite::new(
                        pos,
                        (32.0, 32.0),
                        (
                            props.frames[idx].0 as f32 * 32.0,
                            props.frames[idx].1 as f32 * 32.0,
                            32.0,
                            32.0,
                        ),
                    );
                    sprite.tint = [
                        tint,
                        tint,
                        tint,
                        1.0 - (props.transparency as f32
                            / (level_data::consts::CLEAR as f32 - f32::EPSILON)),
                    ];
                    sprites.push(sprite);
                };

                if let Some(background) = tile.background.0 {
                    drawer(&level_data.tile_properties[background as usize], false);
                }
                if let Some(foreground) = tile.foreground.0 {
                    drawer(&level_data.tile_properties[foreground as usize], true);
                }
                use level_data::consts::*;
                if tile.light != LIGHT_NORMAL {
                    let color = if tile.light < LIGHT_NORMAL { 1.0 } else { 0.0 };
                    let percent = if tile.light < LIGHT_NORMAL {
                        1.0 - (tile.light as f32 / LIGHT_NORMAL as f32)
                    } else if tile.light > LIGHT_NORMAL {
                        (tile.light - LIGHT_NORMAL) as f32 / (LIGHT_BLACK - LIGHT_NORMAL) as f32
                    } else {
                        0.0
                    };
                    sprites.push(Sprite::new_pure_color(
                        pos,
                        (32.0, 32.0),
                        (color, color, color, percent),
                    ));
                }
            }
        }

        if let Tool::Fill = editor_data.tool {
            if (response.ctx.input().pointer.primary_clicked()
                || response.ctx.input().pointer.secondary_clicked())
                && !self.grabbing_resize
                && response.hovered()
            {
                editor_data
                    .cells_history
                    .push((level_data.cells.clone(), level_data.spawners.clone()));
                if editor_data.cells_history.len() > 512 {
                    editor_data.cells_history.remove(0);
                }

                // Fill in with the first selected thing
                let primary_clicked = response.ctx.input().pointer.primary_clicked();
                let set_tile = if editor_data.selected_tiles.len() > 0 {
                    TileId(Some(editor_data.selected_tiles[0] as u16))
                } else {
                    TileId(None)
                };
                let need_tile = if editor_data.foreground_placing {
                    if primary_clicked {
                        level_data.cells.get_cell(mx, my).foreground
                    } else {
                        level_data.cells.get_cell(mx, my).background
                    }
                } else {
                    if primary_clicked {
                        level_data.cells.get_cell(mx, my).background
                    } else {
                        level_data.cells.get_cell(mx, my).foreground
                    }
                };

                let mut queue = VecDeque::new();
                queue.push_back((mx, my));
                while !queue.is_empty() && need_tile != set_tile {
                    let top = *queue.front().unwrap();
                    queue.pop_front();

                    let cell = level_data.cells.get_cell(top.0, top.1);
                    let inside = if editor_data.foreground_placing {
                        if primary_clicked {
                            cell.foreground == need_tile
                        } else {
                            cell.background == need_tile
                        }
                    } else {
                        if primary_clicked {
                            cell.background == need_tile
                        } else {
                            cell.foreground == need_tile
                        }
                    };

                    if inside
                        && top.0 >= 0
                        && top.0 < level_data.cells.width() as i32
                        && top.1 >= 0
                        && top.1 < level_data.cells.height() as i32
                    {
                        // change this and add it!
                        let cell = level_data.cells.get_cell_mut(top.0, top.1);
                        //cell.foreground = set_tile;
                        if editor_data.foreground_placing {
                            if primary_clicked {
                                cell.foreground = set_tile;
                            } else {
                                cell.background = set_tile;
                            }
                        } else {
                            if primary_clicked {
                                cell.background = set_tile;
                            } else {
                                cell.foreground = set_tile;
                            }
                        }
                        queue.push_back((top.0 - 1, top.1));
                        queue.push_back((top.0 + 1, top.1));
                        queue.push_back((top.0, top.1 - 1));
                        queue.push_back((top.0, top.1 + 1));
                    }
                }
            }

            if let Some(tile) = editor_data.selected_tiles.get(0) {
                let props = &level_data.tile_properties[*tile];
                let idx = (editor_data.time_past.as_secs_f32() * (30.0 / props.anim_speed.0 as f32))
                    as usize
                    % props.frames.len();
                let mut sprite = Sprite::new(
                    (mx as f32 * 32.0, my as f32 * 32.0, 0.0),
                    (32.0, 32.0),
                    (
                        props.frames[idx].0 as f32 * 32.0,
                        props.frames[idx].1 as f32 * 32.0,
                        32.0,
                        32.0,
                    ),
                );
                sprite.tint = [0.8, 0.8, 0.8, 0.7];
                sprites.push(sprite);
            }
        }

        if let Tool::TilePicker = editor_data.tool {
            if !self.grabbing_resize && response.hovered() {
                if editor_data.foreground_placing {
                    if response.ctx.input().pointer.primary_clicked() {
                        if let Some(id) = level_data.cells.get_cell(mx, my).foreground.0 {
                            editor_data.selected_tiles = vec![id as usize];
                        } else {
                            editor_data.selected_tiles = vec![];
                        }
                    } else if response.ctx.input().pointer.secondary_clicked() {
                        if let Some(id) = level_data.cells.get_cell(mx, my).background.0 {
                            editor_data.selected_tiles = vec![id as usize];
                        } else {
                            editor_data.selected_tiles = vec![];
                        }
                    }
                } else {
                    if response.ctx.input().pointer.primary_clicked() {
                        if let Some(id) = level_data.cells.get_cell(mx, my).background.0 {
                            editor_data.selected_tiles = vec![id as usize];
                        } else {
                            editor_data.selected_tiles = vec![];
                        }
                    } else if response.ctx.input().pointer.secondary_clicked() {
                        if let Some(id) = level_data.cells.get_cell(mx, my).foreground.0 {
                            editor_data.selected_tiles = vec![id as usize];
                        } else {
                            editor_data.selected_tiles = vec![];
                        }
                    }
                }
            }

            if let Some(tile) = editor_data.selected_tiles.get(0) {
                let props = &level_data.tile_properties[*tile];
                let idx = (editor_data.time_past.as_secs_f32() * (30.0 / props.anim_speed.0 as f32))
                    as usize
                    % props.frames.len();
                let mut sprite = Sprite::new(
                    (mx as f32 * 32.0, my as f32 * 32.0, 0.0),
                    (32.0, 32.0),
                    (
                        props.frames[idx].0 as f32 * 32.0,
                        props.frames[idx].1 as f32 * 32.0,
                        32.0,
                        32.0,
                    ),
                );
                sprite.tint = [0.8, 0.8, 0.8, 0.7];
                sprites.push(sprite);
            }
        }

        if let Tool::Brush = editor_data.tool {
            if tile_placing_enabled
                && (response.ctx.input().pointer.primary_clicked()
                    || response.ctx.input().pointer.secondary_clicked())
            {
                editor_data
                    .cells_history
                    .push((level_data.cells.clone(), level_data.spawners.clone()));
                if editor_data.cells_history.len() > 512 {
                    editor_data.cells_history.remove(0);
                }
            }

            if let Some(ref viewer_selection) = editor_data.viewer_selection.as_ref() {
                let ox = self.brush_origin.0
                    + ((mx - self.brush_origin.0) as f32 / viewer_selection.width() as f32).floor()
                        as i32
                        * viewer_selection.width() as i32;
                let oy = self.brush_origin.1
                    + ((my - self.brush_origin.1) as f32 / viewer_selection.height() as f32).floor()
                        as i32
                        * viewer_selection.height() as i32;

                for y in 0..viewer_selection.height() as i32 {
                    for x in 0..viewer_selection.width() as i32 {
                        let mut drawer = |props: &TileProperties| {
                            let idx = (editor_data.time_past.as_secs_f32()
                                * (30.0 / props.anim_speed.0 as f32))
                                as usize
                                % props.frames.len();
                            let mut sprite = Sprite::new(
                                ((ox + x) as f32 * 32.0, (oy + y) as f32 * 32.0, 0.0),
                                (32.0, 32.0),
                                (
                                    props.frames[idx].0 as f32 * 32.0,
                                    props.frames[idx].1 as f32 * 32.0,
                                    32.0,
                                    32.0,
                                ),
                            );
                            sprite.tint = [0.8, 0.8, 0.8, 0.7];
                            sprites.push(sprite);
                        };

                        if self.copy_selection == None {
                            if let Some(tile) = viewer_selection.get_cell(x, y).foreground.0 {
                                drawer(&level_data.tile_properties[tile as usize]);
                            }
                            if let Some(tile) = viewer_selection.get_cell(x, y).background.0 {
                                drawer(&level_data.tile_properties[tile as usize]);
                            }
                        }

                        if tile_placing_enabled && self.copy_selection == None {
                            let dst = level_data.cells.get_cell_mut(ox + x, oy + y);
                            let src = viewer_selection.get_cell(x, y);
                            if mx == ox + x && my == oy + y {
                                if response.ctx.input().pointer.primary_down() && response.hovered()
                                {
                                    if editor_data.foreground_placing {
                                        dst.foreground = src.foreground;
                                    } else {
                                        dst.background = src.background;
                                    }
                                }
                                if response.ctx.input().pointer.secondary_down()
                                    && response.hovered()
                                {
                                    if editor_data.foreground_placing {
                                        dst.background = src.background;
                                    } else {
                                        dst.foreground = src.foreground;
                                    }
                                }
                            }
                            if editor_data.has_copied_tiles && response.hovered() {
                                dst.foreground = src.foreground;
                                dst.background = src.background;
                            }
                        }
                    }
                }
            }

            if let Some(copy_selection) = self.copy_selection {
                sprites.push(Sprite::new_pure_color(
                    (
                        copy_selection.0 as f32 * 32.0,
                        copy_selection.1 as f32 * 32.0,
                        0.0,
                    ),
                    (
                        (copy_selection.2 - copy_selection.0 + 1) as f32 * 32.0,
                        (copy_selection.3 - copy_selection.1 + 1) as f32 * 32.0,
                    ),
                    (1.0, 1.0, 0.0, 0.4),
                ));
            }
        }
    }

    fn show_grid(
        &mut self,
        sprites: &mut Vec<Sprite>,
        ui: &mut egui::Ui,
        level_data: &mut level_data::LevelData,
        editor_data: &mut EditorData,
        _rect: &egui::Rect,
        response: &egui::Response,
        pointer_pos: &egui::Pos2,
    ) {
        let grid_size = if matches!(editor_data.tool, Tool::Spawners) {
            editor_data.spawner_grid_size
        } else {
            32.0
        };
        let mut max_width = level_data.cells.width() as f32 * 32.0;
        let mut max_height = level_data.cells.height() as f32 * 32.0;
        let (mut min_x, mut min_y) = (0.0, 0.0);
        let cam_size = self.camera.get_proj_size_world_space();
        let grab_size = cam_size.y / (32.0 * 4.0);

        if !matches!(editor_data.tool, Tool::Spawners) {
            if self.resizing_bounds.0
                || self.resizing_bounds.1
                || self.resizing_bounds.2
                || self.resizing_bounds.3
            {
                if self.resizing_bounds.0 {
                    max_height = pointer_pos.y;
                    if max_height - 64.0 < min_y {
                        max_height = min_y + 64.0;
                    }
                    if response.drag_released() {
                        editor_data
                            .cells_history
                            .push((level_data.cells.clone(), level_data.spawners.clone()));
                        if editor_data.cells_history.len() > 512 {
                            editor_data.cells_history.remove(0);
                        }
                        level_data
                            .cells
                            .resize(level_data.cells.width(), max_height as usize / 32);
                    }
                }
                if self.resizing_bounds.1 {
                    min_y = pointer_pos.y;
                    if min_y + 64.0 > max_height {
                        min_y = max_height - 64.0;
                    }
                    if response.drag_released() {
                        editor_data
                            .cells_history
                            .push((level_data.cells.clone(), level_data.spawners.clone()));
                        if editor_data.cells_history.len() > 512 {
                            editor_data.cells_history.remove(0);
                        }
                        let src_offset = min_y as i32 / 32;

                        for spawner in level_data.spawners.iter_mut() {
                            spawner.pos.1 -= (src_offset * 32) as f32;
                        }

                        let new_height = level_data.cells.height() as i32 - src_offset;
                        let mut cells = Cells::new(level_data.cells.width(), new_height as usize);
                        level_data.cells.paste(
                            &mut cells,
                            (0, src_offset),
                            (level_data.cells.width() as i32 - 1, new_height + src_offset),
                            (0, 0),
                        );
                        level_data.cells = cells;
                    }
                }
                if self.resizing_bounds.2 {
                    min_x = pointer_pos.x;
                    if min_x + 64.0 > max_width {
                        min_x = max_width - 64.0;
                    }
                    if response.drag_released() {
                        editor_data
                            .cells_history
                            .push((level_data.cells.clone(), level_data.spawners.clone()));
                        if editor_data.cells_history.len() > 512 {
                            editor_data.cells_history.remove(0);
                        }
                        let src_offset = min_x as i32 / 32;

                        for spawner in level_data.spawners.iter_mut() {
                            spawner.pos.0 -= (src_offset * 32) as f32;
                        }

                        let new_width = level_data.cells.width() as i32 - src_offset;
                        let mut cells = Cells::new(new_width as usize, level_data.cells.height());
                        level_data.cells.paste(
                            &mut cells,
                            (src_offset, 0),
                            (new_width + src_offset, level_data.cells.height() as i32 - 1),
                            (0, 0),
                        );
                        level_data.cells = cells;
                    }
                }
                if self.resizing_bounds.3 {
                    max_width = pointer_pos.x;
                    if max_width - 64.0 < min_x {
                        max_width = min_x + 64.0;
                    }
                    if response.drag_released() {
                        editor_data
                            .cells_history
                            .push((level_data.cells.clone(), level_data.spawners.clone()));
                        if editor_data.cells_history.len() > 512 {
                            editor_data.cells_history.remove(0);
                        }
                        level_data
                            .cells
                            .resize(max_width as usize / 32, level_data.cells.height());
                    }
                }

                if response.drag_released() {
                    self.resizing_bounds = (false, false, false, false);
                }
            } else {
                if egui::Rect::from_x_y_ranges(
                    0.0..=max_width,
                    max_height - grab_size..=max_height + grab_size,
                )
                .contains(*pointer_pos)
                {
                    ui.output().cursor_icon = egui::CursorIcon::ResizeVertical;
                    self.grabbing_resize = true;
                    if response.drag_started() {
                        self.resizing_bounds = (true, false, false, false);
                    }
                } else if egui::Rect::from_x_y_ranges(0.0..=max_width, -grab_size..=grab_size)
                    .contains(*pointer_pos)
                {
                    ui.output().cursor_icon = egui::CursorIcon::ResizeVertical;
                    self.grabbing_resize = true;
                    if response.drag_started() {
                        self.resizing_bounds = (false, true, false, false);
                    }
                } else if egui::Rect::from_x_y_ranges(-grab_size..=grab_size, 0.0..=max_height)
                    .contains(*pointer_pos)
                {
                    ui.output().cursor_icon = egui::CursorIcon::ResizeHorizontal;
                    self.grabbing_resize = true;
                    if response.drag_started() {
                        self.resizing_bounds = (false, false, true, false);
                    }
                } else if egui::Rect::from_x_y_ranges(
                    max_width - grab_size..=max_width + grab_size,
                    0.0..=max_height,
                )
                .contains(*pointer_pos)
                {
                    ui.output().cursor_icon = egui::CursorIcon::ResizeHorizontal;
                    self.grabbing_resize = true;
                    if response.drag_started() {
                        self.resizing_bounds = (false, false, false, true);
                    }
                } else {
                    self.grabbing_resize = false;
                }
            }
        }

        if self.show_grid {
            let top_left = self.camera.get_top_left_world_space();
            let start = cgmath::vec2(
                (top_left.x / grid_size).floor() as i32,
                (top_left.y / grid_size).floor() as i32,
            );
            let end = cgmath::vec2(
                (self.camera.get_bottom_right_world_space().x / grid_size).ceil() as i32 + 1,
                (self.camera.get_bottom_right_world_space().y / grid_size).ceil() as i32 + 1,
            );

            let spawner_tool = matches!(editor_data.tool, Tool::Spawners);
            let alpha = if spawner_tool {
                ((self.camera.zoom - 0.02) / (5.0 - 0.02)).clamp(0.0, 0.25)
            } else {
                ((self.camera.zoom - 0.02) / (1.0 - 0.02)).clamp(0.0, 0.75)
            };

            if grid_size > 1.0 {
                for column in start.x..=end.x {
                    sprites.push(Sprite::new_pure_color(
                        (column as f32 * grid_size, top_left.y, 0.0),
                        (cam_size.y / (32.0 * 15.0), cam_size.y),
                        (1.0, 1.0, 1.0, alpha),
                    ));
                }

                for row in start.y..=end.y {
                    sprites.push(Sprite::new_pure_color(
                        (top_left.x, row as f32 * grid_size, 0.0),
                        (cam_size.x, cam_size.y / (32.0 * 15.0)),
                        (1.0, 1.0, 1.0, alpha),
                    ));
                }
            }
        }

        if self.show_original_screen_size {
            sprites.append(
                &mut (Sprite::new_rect(
                    (
                        self.camera.pos.x - (320.0 / 2.0),
                        self.camera.pos.y - (240.0 / 2.0),
                    ),
                    (
                        self.camera.pos.x + (320.0 / 2.0),
                        self.camera.pos.y + (240.0 / 2.0),
                    ),
                    cam_size.y / (32.0 * 8.0),
                    (0.1, 0.3, 0.8, 0.7),
                ))
                .to_vec(),
            );
        }

        if !matches!(editor_data.tool, Tool::Spawners) {
            sprites.append(
                &mut (Sprite::new_rect(
                    (min_x, min_y),
                    (max_width, max_height),
                    cam_size.y / (32.0 * 10.0),
                    (0.1, 0.3, 0.8, 0.7),
                ))
                .to_vec(),
            );
        }
    }
    fn update_level_spawners(
        &mut self,
        sprites: &mut Vec<Sprite>,
        ui: &mut egui::Ui,
        level_data: &mut level_data::LevelData,
        rect: &egui::Rect,
        response: &egui::Response,
        editor_data: &mut EditorData,
        pointer_pos: &egui::Pos2,
    ) {
        // let top_left = self.camera.get_top_left_world_space();
        let cam_size = self.camera.get_proj_size_world_space();
        let mut delete_idx = None;
        let mut hovered_spawners = Vec::new();
        for (idx, spawner) in level_data.spawners.iter_mut().enumerate() {
            draw_spawner(
                sprites,
                spawner,
                &self.camera,
                editor_data,
                matches!(editor_data.tool, Tool::Spawners),
            );
            let spawner_rect = get_spawner_size(spawner);
            if pointer_pos.x > spawner.pos.0
                && pointer_pos.x < spawner.pos.0 + spawner_rect.0
                && pointer_pos.y > spawner.pos.1
                && pointer_pos.y < spawner.pos.1 + spawner_rect.1
                && matches!(editor_data.tool, Tool::Spawners)
            {
                hovered_spawners.push(idx);
            }
            if editor_data.selected_spawner == Some(idx)
                && matches!(editor_data.tool, Tool::Spawners)
            {
                sprites.push(Sprite::new_pure_color(
                    (spawner.pos.0, spawner.pos.1, 0.0),
                    (spawner_rect.0, spawner_rect.1),
                    (0.0, 0.2, 1.0, 0.5),
                ));
            }
        }
        if !matches!(editor_data.tool, Tool::Spawners) {
            self.close_context_menu = false;
            return;
        }
        let duplicated_spawners = level_data.spawners.clone();
        let mut populated = false;
        hovered_spawners.sort_by(|idx_a, idx_b| {
            let a = get_spawner_size(&level_data.spawners[*idx_a]);
            let b = get_spawner_size(&level_data.spawners[*idx_b]);
            let cmp = (a.0 + a.1).total_cmp(&(b.0 + b.1));
            if cmp.is_eq() {
                idx_b.cmp(idx_a)
            } else {
                cmp
            }
        });
        if let Some(idx) = hovered_spawners.get(0) {
            //let spawner = &mut level_data.spawners[*idx];
            if response.clicked() {
                editor_data.selected_spawner = Some(*idx);
                self.right_clicked_spawner_idx = None;
            }
            if response.ctx.input().pointer.secondary_clicked() {
                self.right_clicked_spawner_idx = Some(*idx);
            }
        } else if response.hovered() {
            if response.clicked() {
                editor_data.selected_spawner = None;
                self.right_clicked_spawner_idx = None;
            }
            if response.double_clicked_by(egui::PointerButton::Primary) {
                editor_data
                    .cells_history
                    .push((level_data.cells.clone(), duplicated_spawners.clone()));
                let mut spawner = editor_data.spawner_template.clone();
                spawner.dropped_item = None;
                spawner.spawner_group = None;
                spawner.spawning_criteria = SpawningCriteria {
                    spawn_delay_secs: 0.0,
                    mode: cnmo_parse::lparse::level_data::cnms_types::SpawnerMode::MultiAndSingleplayer,
                    max_concurrent_spawns: 0,
                };
                if editor_data.spawner_grid_size > 1.0 {
                    spawner.pos.0 = (pointer_pos.x / editor_data.spawner_grid_size).round()
                        * editor_data.spawner_grid_size;
                    spawner.pos.1 = (pointer_pos.y / editor_data.spawner_grid_size).round()
                        * editor_data.spawner_grid_size;
                } else {
                    spawner.pos.0 = pointer_pos.x;
                    spawner.pos.1 = pointer_pos.y;
                }
                level_data.spawners.push(spawner);
                editor_data.selected_spawner = Some(level_data.spawners.len() - 1);
            }
            if response.ctx.input().pointer.secondary_clicked() {
                self.right_clicked_spawner_idx = None;
            }
        }
        if let Some(idx) = editor_data.selected_spawner {
            let spawner = &mut level_data.spawners[idx];
            if response.hovered() {
                if response.drag_started() {
                    editor_data
                        .cells_history
                        .push((level_data.cells.clone(), duplicated_spawners.clone()));
                }
                if response.dragged_by(egui::PointerButton::Primary) {
                    spawner.pos.0 += (response.drag_delta().x / rect.width()) * cam_size.x;
                    spawner.pos.1 += (response.drag_delta().y / rect.height()) * cam_size.y;
                }
                if response.drag_released() && editor_data.spawner_grid_size > 1.0 {
                    spawner.pos.0 = (spawner.pos.0 / editor_data.spawner_grid_size).round()
                        * editor_data.spawner_grid_size;
                    spawner.pos.1 = (spawner.pos.1 / editor_data.spawner_grid_size).round()
                        * editor_data.spawner_grid_size;
                }
                if response.ctx.input().key_pressed(egui::Key::Delete)
                    || (response.ctx.input().key_pressed(egui::Key::D)
                        && response.ctx.input().modifiers.alt)
                {
                    delete_idx = Some(idx);
                }
            }
            if response.ctx.input().key_pressed(egui::Key::C)
                && (response.ctx.input().modifiers.ctrl || response.ctx.input().modifiers.mac_cmd)
                && editor_data.editing_text == None
            {
                editor_data.spawner_template = spawner.clone();
                log::info!("Copied Spawner Data");
            }
            if response.ctx.input().key_pressed(egui::Key::V)
                && response.ctx.input().modifiers.shift
                && response.hovered()
                && editor_data.editing_text == None
            {
                editor_data
                    .cells_history
                    .push((level_data.cells.clone(), duplicated_spawners.clone()));
                spawner.dropped_item = editor_data.spawner_template.dropped_item.clone();
                spawner.spawning_criteria = editor_data.spawner_template.spawning_criteria.clone();
                spawner.type_data = editor_data.spawner_template.type_data.clone();
                spawner.spawner_group = editor_data.spawner_template.spawner_group.clone();
                log::info!("Pasted Spawner Data in Place");
            }
        }
        if let Some(idx) = editor_data.selected_spawner {
            let spawner = &level_data.spawners[idx];
            let mut location = None;
            if let WobjType::Teleport(teleport) = &spawner.type_data {
                populated |= true;
                location = Some(teleport.loc);
            }
            if let WobjType::TeleportArea1 { loc, .. } = &spawner.type_data {
                populated |= true;
                location = Some(*loc);
            }
            if let WobjType::TeleportArea2 { loc, .. } = &spawner.type_data {
                populated |= true;
                location = Some(*loc);
            }
            if let Some(loc) = location {
                sprites.push(Sprite::new(
                    (loc.0 - 16.0, loc.1 - 16.0, 0.0),
                    (16.0, 16.0),
                    (256.0, 1088.0, 16.0, 16.0),
                ));
                sprites.push(Sprite::new(
                    (loc.0, loc.1 - 16.0, 0.0),
                    (16.0, 16.0),
                    (256.0 + 16.0, 1088.0, -16.0, 16.0),
                ));
                sprites.push(Sprite::new(
                    (loc.0 - 16.0, loc.1, 0.0),
                    (16.0, 16.0),
                    (256.0, 1088.0 + 16.0, 16.0, -16.0),
                ));
                sprites.push(Sprite::new(
                    (loc.0, loc.1, 0.0),
                    (16.0, 16.0),
                    (256.0 + 16.0, 1088.0 + 16.0, -16.0, -16.0),
                ));
            }
        }
        if ((response.ctx.input().key_pressed(egui::Key::V) && (response.ctx.input().modifiers.ctrl || response.ctx.input().modifiers.mac_cmd) && editor_data.editing_text == None)
            || (ui.ctx().input().key_pressed(egui::Key::Space) && editor_data.editing_text == None)) && response.hovered()
        {
            editor_data
                .cells_history
                .push((level_data.cells.clone(), level_data.spawners.clone()));
            let mut spawner = editor_data.spawner_template.clone();
            if editor_data.spawner_grid_size > 1.0 {
                spawner.pos.0 = (pointer_pos.x / editor_data.spawner_grid_size).round()
                    * editor_data.spawner_grid_size;
                spawner.pos.1 = (pointer_pos.y / editor_data.spawner_grid_size).round()
                    * editor_data.spawner_grid_size;
            } else {
                spawner.pos.0 = pointer_pos.x;
                spawner.pos.1 = pointer_pos.y;
            }
            level_data.spawners.push(spawner);
            editor_data.selected_spawner = Some(level_data.spawners.len() - 1);
        }
        populated |= self.right_clicked_spawner_idx != None;
        self.hovered_on_context_menu = false;
        if populated {
            let response = response.clone().context_menu(|ui| {
                if let Some(idx) = self.right_clicked_spawner_idx {
                    if ui.button("Delete spawner").clicked() {
                        editor_data
                            .cells_history
                            .push((level_data.cells.clone(), level_data.spawners.clone()));
                        editor_data.selected_spawner = None;
                        self.right_clicked_spawner_idx = None;
                        level_data.spawners.remove(idx);
                        ui.close_menu();
                    }
                }
                if let Some(idx) = editor_data.selected_spawner {
                    let spawner = &mut level_data.spawners[idx];
                    match &mut spawner.type_data {
                        WobjType::Teleport(
                            cnmo_parse::lparse::level_data::cnms_types::wobj_type::Teleport {
                                loc,
                                ..
                            },
                        )
                        | WobjType::TeleportArea1 { loc, .. }
                        | WobjType::TeleportArea2 { loc, .. } => {
                            if ui.button("Set teleport location").clicked() {
                                editor_data
                                    .cells_history
                                    .push((level_data.cells.clone(), duplicated_spawners.clone()));
                                loc.0 = pointer_pos.x;
                                loc.1 = pointer_pos.y;
                                ui.close_menu();
                            }
                        }
                        _ => {}
                    }
                }
                if self.close_context_menu {
                    ui.close_menu();
                    self.close_context_menu = false;
                }
            });
            self.close_context_menu = response.lost_focus();
            self.hovered_on_context_menu = response.hovered();
        }
        if let Some(idx) = delete_idx {
            editor_data
                .cells_history
                .push((level_data.cells.clone(), level_data.spawners.clone()));
            level_data.spawners.remove(idx);
            editor_data.selected_spawner = None;
        }
    }
}

fn get_spawner_size(spawner: &cnmo_parse::lparse::level_data::cnms_types::Spawner) -> (f32, f32) {
    use cnmo_parse::lparse::level_data::cnms_types::wobj_type::{
        BackgroundSwitcherShape, PushZoneType, RockGuyType, RuneType, TunesTriggerSize,
        UpgradeTriggerType,
    };
    match spawner.type_data {
        WobjType::TeleportArea1 { .. } | WobjType::TeleportArea2 { .. } | WobjType::Dragon { .. } | WobjType::SuperDragon { .. } => {
            (128.0, 128.0)
        }
        WobjType::Bozo { mark_ii } => {
            if mark_ii {
                (48.0, 64.0)
            } else {
                (64.0, 128.0)
            }
        }
        WobjType::TunesTrigger { size, .. } => match size {
            TunesTriggerSize::Small => (32.0, 32.0),
            TunesTriggerSize::Big => (64.0, 64.0),
            TunesTriggerSize::VeryBig => (96.0, 96.0),
        },
        WobjType::BackgroundSwitcher { shape, .. } => match shape {
            BackgroundSwitcherShape::Small => (32.0, 32.0),
            BackgroundSwitcherShape::Horizontal => (128.0, 32.0),
            BackgroundSwitcherShape::Vertical => (32.0, 96.0),
        },
        WobjType::WandRune { rune_type } => match rune_type {
            RuneType::Ice => (46.0, 44.0),
            RuneType::Air => (64.0, 32.0),
            RuneType::Fire => (64.0, 64.0),
            RuneType::Lightning => (64.0, 64.0),
        },
        WobjType::UpgradeTrigger { trigger_type } => match trigger_type {
            UpgradeTriggerType::DeephausBoots => (32.0, 32.0),
            UpgradeTriggerType::Wings => (36.0, 38.0),
            UpgradeTriggerType::CrystalWings => (48.0, 48.0),
            UpgradeTriggerType::None => (32.0, 32.0),
            UpgradeTriggerType::Vortex => (32.0, 32.0),
            UpgradeTriggerType::MaxPowerRune { .. } => (48.0, 48.0),
        },
        WobjType::Heavy { .. } => (64.0, 64.0),
        WobjType::BozoPin { .. } => (48.0, 64.0),
        WobjType::LavaMonster { .. } => (48.0, 48.0),
        WobjType::TtMinion { small } => {
            if small {
                (32.0, 32.0)
            } else {
                (32.0, 64.0)
            }
        }
        WobjType::SlimeWalker | WobjType::LavaDragonHead { .. } => (64.0, 64.0),
        WobjType::EaterBug { .. } => (32.0, 96.0),
        WobjType::BozoLaserMinion { .. } => (32.0, 64.0),
        WobjType::PushZone { push_zone_type, .. } => match push_zone_type {
            PushZoneType::Horizontal => (128.0, 128.0),
            PushZoneType::Vertical => (64.0, 64.0),
            PushZoneType::HorizontalSmall => (32.0, 32.0),
        },
        WobjType::VerticalWindZone { .. } => (64.0, 64.0),
        WobjType::Jumpthrough { big } => {
            if big {
                (96.0, 32.0)
            } else {
                (32.0, 32.0)
            }
        }
        WobjType::RockGuy { rock_guy_type } => match rock_guy_type {
            RockGuyType::Medium => (32.0, 64.0),
            RockGuyType::Small1 => (22.0, 19.0),
            RockGuyType::Small2 { .. } => (14.0, 14.0),
        },
        WobjType::HealthSetTrigger { .. } => (64.0, 96.0),
        WobjType::Vortex { .. } => (96.0, 96.0),
        WobjType::RockGuySmasher | WobjType::TeleportTrigger1 { .. } => (32.0, 96.0),
        WobjType::RockGuySlider | WobjType::Wolf => (64.0, 32.0),
        WobjType::Supervirus => (48.0, 96.0),
        _ => (32.0, 32.0),
    }
}
fn draw_spawner(
    sprites: &mut Vec<Sprite>,
    spawner: &cnmo_parse::lparse::level_data::cnms_types::Spawner,
    camera: &Camera,
    editor_data: &EditorData,
    active: bool,
) {
    use cnmo_parse::lparse::level_data::cnms_types::{
        item_type::ItemType,
        wobj_type::{
            BackgroundSwitcherShape, KeyColor, PushZoneType, RockGuyType, RuneType, TtNodeType,
            TunesTriggerSize, UpgradeTriggerType,
        },
    };

    let _cam_size = camera.get_proj_size_world_space();
    let mut draw_rect = |x: i32, y: i32, w: i32, h: i32| {
        let mut sprite = Sprite::new(
            (spawner.pos.0, spawner.pos.1, 0.0),
            (w.abs() as f32, h as f32),
            (x as f32, y as f32, w as f32, h as f32),
        );
        sprite.tint[3] = if active { 1.0 } else { 0.6 };
        sprites.push(sprite);
    };
    let draw_moving = |sprites: &mut Vec<Sprite>, dist: f32, speed: f32, vertical: bool| {
        let dist = if dist > -0.01 && dist < 0.01 { f32::EPSILON } else { dist };
        let unclamped_pos = editor_data.time_past.as_secs_f32() * (speed * dist.signum() * 30.0);
        let pos = if (unclamped_pos / dist.abs()) as i32 % 2 == 0 {
            unclamped_pos.rem_euclid(dist.abs())
        } else {
            dist.abs() - unclamped_pos.rem_euclid(dist.abs())
        };
        let srcpos = if dist < 0.0 { 
            level_data::Point(
                spawner.pos.0 + if vertical { 0.0 } else { dist },
                spawner.pos.1 + if vertical { dist } else { 0.0 },
            )
        } else { spawner.pos };
        if vertical {
            sprites.push(Sprite::new_pure_color(
                (spawner.pos.0 + 16.0, spawner.pos.1 + 16.0, 0.0),
                (3.0, dist),
                (1.0, 1.0, 0.0, 0.9),
            ));
            sprites.append(
                &mut Sprite::new_rect(
                    (srcpos.0, pos + srcpos.1),
                    (srcpos.0 + 32.0, pos + srcpos.1 + 32.0),
                    3.0,
                    (0.0, 0.2, 1.0, 0.8),
                )
                .to_vec(),
            );
        } else {
            sprites.push(Sprite::new_pure_color(
                (spawner.pos.0 + 16.0, spawner.pos.1 + 16.0, 0.0),
                (dist, 3.0),
                (1.0, 1.0, 0.0, 0.9),
            ));
            sprites.append(
                &mut Sprite::new_rect(
                    (pos + srcpos.0, srcpos.1),
                    (pos + srcpos.0 + 32.0, srcpos.1 + 32.0),
                    3.0,
                    (0.0, 0.2, 1.0, 0.8),
                )
                .to_vec(),
            );
        }
    };

    match &spawner.type_data {
        WobjType::Teleport(tele) => {
            draw_rect(96, 32, 32, 32);
            let tele_string = if tele.cost == 0 {
                tele.name.clone()
            } else {
                format!("{}: ${}", tele.name, tele.cost)
            };
            Sprite::create_string(
                spawner.pos.0 - (tele_string.len() as f32 * 2.0) + 16.0,
                spawner.pos.1 - 5.0,
                4.0,
                &tele_string,
                sprites,
            );
        }
        WobjType::Slime { flying } => {
            if *flying {
                draw_rect(448, 320, 32, 32);
            } else {
                draw_rect(32, 256, 32, 32);
            }
        }
        WobjType::TunesTrigger { size, music_id } => {
            let half_width = match size {
                TunesTriggerSize::Small => {
                    draw_rect(416, 1216, 32, 32);
                    16.0
                }
                TunesTriggerSize::Big => {
                    draw_rect(448, 1216, 64, 64);
                    32.0
                }
                TunesTriggerSize::VeryBig => {
                    draw_rect(416, 2848, 96, 96);
                    48.0
                }
            };
            let id_string = format!("Music ID: {}", *music_id);
            Sprite::create_string(
                spawner.pos.0 - (id_string.len() as f32 * 2.0) + half_width,
                spawner.pos.1 - 5.0,
                4.0,
                &id_string,
                sprites,
            );
        }
        WobjType::TextSpawner { dialoge_box, .. } => {
            if *dialoge_box {
                draw_rect(288, 1344, 32, 32);
            } else {
                draw_rect(352, 1216, 32, 32);
            }
        }
        WobjType::MovingPlatform {
            vertical,
            dist,
            speed,
        } => {
            draw_rect(288, 160, 32, 32);
            draw_moving(sprites, *dist, *speed, *vertical);
        }
        WobjType::MovingFire {
            vertical,
            dist,
            speed,
            despawn: _,
        } => {
            draw_rect(480, 2048, 32, 32);
            draw_moving(sprites, *dist as f32, *speed, *vertical);
        }
        WobjType::BreakableWall { skin_id, .. } => match skin_id {
            &Some(0) => draw_rect(288, 1312, 32, 32),
            &Some(1) => draw_rect(256, 1248, 32, 32),
            &Some(2) => draw_rect(288, 1248, 32, 32),
            &Some(3) => draw_rect(256, 1280, 32, 32),
            &Some(4) => draw_rect(288, 1280, 32, 32),
            &Some(5) => draw_rect(256, 1312, 32, 32),
            _ => draw_rect(256, 160, 32, 32),
        },
        WobjType::BackgroundSwitcher { shape, .. } => match shape {
            &BackgroundSwitcherShape::Small => draw_rect(352, 1312, 32, 32),
            &BackgroundSwitcherShape::Horizontal => draw_rect(384, 2816, 128, 32),
            &BackgroundSwitcherShape::Vertical => draw_rect(384, 2848, 32, 96),
        },
        WobjType::DroppedItem { item } => {
            draw_rect(320, 1216, 32, 32);
            match item {
                ItemType::Shotgun => draw_rect(32, 352, 32, 32),
                ItemType::Knife => draw_rect(64, 352, 32, 32),
                ItemType::Apple => draw_rect(96, 352, 32, 32),
                ItemType::Cake => draw_rect(128, 352, 32, 32),
                ItemType::StrengthPotion => draw_rect(160, 352, 32, 32),
                ItemType::SpeedPotion => draw_rect(192, 352, 32, 32),
                ItemType::JumpPotion => draw_rect(224, 352, 32, 32),
                ItemType::Sword => draw_rect(0, 384, 32, 32),
                ItemType::HealthPotion => draw_rect(32, 384, 32, 32),
                ItemType::Sniper => draw_rect(64, 384, 32, 32),
                ItemType::Money50 => draw_rect(96, 384, 32, 32),
                ItemType::Money100 => draw_rect(128, 384, 32, 32),
                ItemType::Money500 => draw_rect(160, 384, 32, 32),
                ItemType::Cheeseburger => draw_rect(192, 384, 32, 32),
                ItemType::GoldenAxe => draw_rect(224, 384, 32, 32),
                ItemType::UnboundWand => draw_rect(0, 416, 32, 32),
                ItemType::FireWand => draw_rect(32, 416, 32, 32),
                ItemType::IceWand => draw_rect(64, 416, 32, 32),
                ItemType::AirWand => draw_rect(96, 416, 32, 32),
                ItemType::LightningWand => draw_rect(128, 416, 32, 32),
                ItemType::GoldenShotgun => draw_rect(160, 416, 32, 32),
                ItemType::LaserRifle => draw_rect(192, 416, 32, 32),
                ItemType::RocketLauncher => draw_rect(224, 416, 32, 32),
                ItemType::FirePotion => draw_rect(0, 448, 32, 32),
                ItemType::Minigun => draw_rect(32, 448, 32, 32),
                ItemType::MegaPotion => draw_rect(64, 448, 32, 32),
                ItemType::UltraMegaPotion => draw_rect(96, 448, 32, 32),
                ItemType::Awp => draw_rect(128, 448, 32, 32),
                ItemType::Flamethrower => draw_rect(160, 448, 32, 32),
                ItemType::PoisionusStrengthPotion => draw_rect(192, 448, 32, 32),
                ItemType::PoisionusSpeedPotion => draw_rect(224, 448, 32, 32),
                ItemType::PoisionusJumpPotion => draw_rect(0, 480, 32, 32),
                ItemType::Beastchurger => draw_rect(32, 480, 32, 32),
                ItemType::UltraSword => draw_rect(64, 480, 32, 32),
                ItemType::HeavyHammer => draw_rect(96, 480, 32, 32),
                ItemType::FissionGun => draw_rect(128, 480, 32, 32),
                ItemType::KeyRed => draw_rect(160, 480, 32, 32),
                ItemType::KeyGreen => draw_rect(192, 480, 32, 32),
                ItemType::KeyBlue => draw_rect(224, 480, 32, 32),
                ItemType::ExtraLifeJuice => draw_rect(128, 736, 32, 32),
                ItemType::Wrench => draw_rect(160, 2048, 32, 32),
            }
        }
        WobjType::WandRune { rune_type } => match rune_type {
            RuneType::Ice => draw_rect(263, 361, 46, 44),
            RuneType::Air => draw_rect(448, 32, 64, 32),
            RuneType::Fire => draw_rect(256, 64, 64, 64),
            RuneType::Lightning => draw_rect(320, 64, 64, 64),
        },
        WobjType::UpgradeTrigger { trigger_type } => match trigger_type {
            UpgradeTriggerType::DeephausBoots => draw_rect(256, 128, 32, 32),
            UpgradeTriggerType::Wings => draw_rect(320, 352, 36, 38),
            UpgradeTriggerType::CrystalWings => draw_rect(384, 352, 48, 48),
            UpgradeTriggerType::None => draw_rect(480, 3936, 32, 32),
            UpgradeTriggerType::Vortex => draw_rect(96, 224, 32, 32),
            UpgradeTriggerType::MaxPowerRune {
                skin_power_override,
            } => {
                draw_rect(192, 1120, 48, 48);
                let text = if let Some(skin) = skin_power_override {
                    format!("override skin: {}", skin)
                } else {
                    "".to_string()
                };
                Sprite::create_string(
                    spawner.pos.0 + 24.0 - (text.len() as f32 * 2.0),
                    spawner.pos.1 - 5.0,
                    4.0,
                    &text,
                    sprites,
                );
            }
        },
        WobjType::Heavy { face_left, .. } => {
            if *face_left {
                draw_rect(128 + 64, 224, -64, 64)
            } else {
                draw_rect(128, 224, 64, 64)
            }
        }
        WobjType::Dragon { space_skin } => {
            if *space_skin {
                draw_rect(256, 640, 128, 128);
            } else {
                draw_rect(192, 224, 128, 128);
            }
        }
        WobjType::BozoPin { .. } => {
            draw_rect(320, 1088, 48, 64);
        }
        WobjType::Bozo { mark_ii } => {
            if *mark_ii {
                draw_rect(224, 2432, 48, 64);
            } else {
                draw_rect(448, 64, 64, 128);
            }
        }
        WobjType::SilverSlime => draw_rect(64, 256, 32, 32),
        WobjType::LavaMonster { face_left } => {
            if *face_left {
                draw_rect(416 + 48, 400, -48, 48);
            } else {
                draw_rect(416, 400, 48, 48);
            }
        }
        WobjType::TtMinion { small } => {
            if *small {
                draw_rect(256, 416, 32, 32);
            } else {
                draw_rect(288, 416, 32, 64);
            }
        }
        WobjType::SlimeWalker => draw_rect(384, 1376, 64, 64),
        WobjType::MegaFish { .. } => draw_rect(256, 448, 32, 32),
        WobjType::LavaDragonHead { .. } => draw_rect(0, 2368, 64, 64),
        WobjType::TtNode { node_type } => match node_type {
            &TtNodeType::ChaseTrigger => draw_rect(320, 1248, 32, 32),
            &TtNodeType::NormalTrigger => draw_rect(320, 1280, 32, 32),
            &TtNodeType::BozoWaypoint => draw_rect(320, 2496, 32, 32),
            &TtNodeType::Waypoint(waypoint_id) => {
                draw_rect(320, 1312, 32, 32);
                Sprite::create_string(
                    spawner.pos.0,
                    spawner.pos.1 - 5.0,
                    4.0,
                    format!("Waypoint ID: {waypoint_id}").as_str(),
                    sprites,
                );
            }
        },
        WobjType::TtBoss { .. } => draw_rect(384, 416, 32, 32),
        WobjType::EaterBug { .. } => draw_rect(352, 1408, 32, 96),
        WobjType::SpiderWalker { .. } => draw_rect(288, 1472, 32, 32),
        WobjType::SpikeTrap => draw_rect(224, 1536, 32, 32),
        WobjType::RotatingFireColunmPiece {
            origin_x,
            degrees_per_second,
        } => {
            draw_rect(416, 2048, 32, 32);
            let dist = spawner.pos.0 - *origin_x as f32;
            let pos_x = *origin_x as f32
                + dist
                    * ((editor_data.time_past.as_secs_f32() * *degrees_per_second).to_radians())
                        .cos();
            let pos_y = spawner.pos.1
                + dist
                    * -((editor_data.time_past.as_secs_f32() * *degrees_per_second).to_radians())
                        .sin();
            sprites.append(
                &mut Sprite::new_rect(
                    (pos_x, pos_y),
                    (pos_x + 32.0, pos_y + 32.0),
                    3.0,
                    (0.7, 0.3, 0.0, 0.8),
                )
                .to_vec(),
            );
        }
        WobjType::SuperDragon { .. } => draw_rect(384, 1664, 128, 128),
        WobjType::SuperDragonLandingZone { .. } => draw_rect(384, 2080, 32, 32),
        WobjType::BozoLaserMinion { .. } => draw_rect(384, 2528, 32, 64),
        WobjType::Checkpoint { .. } => draw_rect(384, 2592, 32, 32),
        WobjType::SpikeGuy => draw_rect(384, 2208, 32, 32),
        WobjType::BanditGuy { .. } => draw_rect(416, 2208, 32, 32),
        WobjType::PushZone { push_zone_type, .. } => match push_zone_type {
            PushZoneType::Horizontal => draw_rect(384, 1856, 128, 128),
            PushZoneType::Vertical => draw_rect(384, 1792, 64, 64),
            PushZoneType::HorizontalSmall => draw_rect(416, 2592, 32, 32),
        },
        WobjType::VerticalWindZone { .. } => draw_rect(384 + 64, 1792, 64, 64),
        WobjType::DisapearingPlatform {
            time_on,
            time_off,
            starts_on,
        } => {
            let mut sprite = Sprite::new(
                (spawner.pos.0, spawner.pos.1, 0.0),
                (32.0, 32.0),
                (352.0, 2112.0, 32.0, 32.0),
            );
            let time = editor_data
                .time_past
                .as_secs_f32()
                .rem_euclid(*time_on + *time_off);
            if *time_on + *time_off > f32::EPSILON {
                if *starts_on {
                    if time > *time_on {
                        sprite.tint = [0.5, 0.5, 0.5, 0.5]
                    }
                } else {
                    if time < *time_off {
                        sprite.tint = [0.5, 0.5, 0.5, 0.5]
                    }
                }
            }
            sprites.push(sprite);
        }
        WobjType::KamakaziSlime => draw_rect(448, 2080, 32, 32),
        WobjType::SpringBoard { jump_velocity } => {
            draw_rect(448, 0, 32, 32);
            sprites.push(Sprite::new_pure_color(
                (spawner.pos.0 + 16.0, spawner.pos.1 + 2.0, 0.0),
                (3.0, -jump_velocity * jump_velocity),
                (1.0, 1.0, 0.0, 0.6),
            ));
        }
        WobjType::Jumpthrough { big } => {
            if *big {
                draw_rect(416, 2944, 96, 32);
            } else {
                draw_rect(384, 2944, 32, 32);
            }
        }
        WobjType::BreakablePlatform { .. } => draw_rect(256, 1184, 32, 32),
        WobjType::LockedBlock { color, .. } => match color {
            &KeyColor::Red => draw_rect(288, 1120, 32, 32),
            &KeyColor::Green => draw_rect(288, 1152, 32, 32),
            &KeyColor::Blue => draw_rect(288, 1184, 32, 32),
        },
        WobjType::RockGuy { rock_guy_type } => match rock_guy_type {
            RockGuyType::Medium => draw_rect(384, 3040, 32, 64),
            RockGuyType::Small1 => draw_rect(390, 3117, 22, 19),
            RockGuyType::Small2 { face_left } => {
                if *face_left {
                    draw_rect(425 + 14, 3122, -14, 14);
                } else {
                    draw_rect(425, 3122, 14, 14);
                }
            }
        },
        WobjType::RockGuySlider => draw_rect(384, 3136, 64, 32),
        WobjType::RockGuySmasher => draw_rect(384, 3392, 32, 96),
        WobjType::HealthSetTrigger { .. } => draw_rect(448, 3968, 64, 96),
        WobjType::Vortex { .. } => draw_rect(256, 3968, 96, 96),
        WobjType::GraphicsChangeTrigger { .. } => draw_rect(256, 1344, 32, 32),
        WobjType::BossBarInfo { .. } => draw_rect(384, 1344, 32, 32),
        WobjType::BgSpeed { vertical_axis, .. } => {
            draw_rect(480, 3392 + (32 * *vertical_axis as i32), 32, 32)
        }
        WobjType::BgTransparency { .. } => draw_rect(480, 3392 + 64, 32, 32),
        WobjType::TeleportTrigger1 { link_id, .. } | WobjType::TeleportArea2 { link_id, .. } | WobjType::TeleportArea1 { link_id, .. } => {
            if matches!(spawner.type_data, WobjType::TeleportArea2 { .. }) {
                draw_rect(384, 4224, 128, 128);
                draw_rect(352, 4288, 32, 32);
            } else if matches!(spawner.type_data, WobjType::TeleportTrigger1 { .. }) {
                draw_rect(480, 3392 + 96, 32, 96);
            } else {
                draw_rect(384, 4224, 128, 128);
            }
            let text = format!("Link ID: {}", *link_id);
            Sprite::create_string(spawner.pos.0, spawner.pos.1 - 5.0, 4.0, &text, sprites);
        }
        WobjType::SfxPoint { .. } => draw_rect(480, 3360, 32, 32),
        WobjType::Wolf => draw_rect(352, 4480, 64, 32),
        WobjType::Supervirus => sprites.push(Sprite::new_pure_color(
            (spawner.pos.0, spawner.pos.1, 0.0),
            (48.0, 96.0),
            (1.0, 0.0, 0.0, 0.8),
        )),
        WobjType::Lua { lua_wobj_type } => {
            draw_rect(448, 1344, 32, 32);
            let text = format!("Lua Type ID: {}", *lua_wobj_type);
            Sprite::create_string(spawner.pos.0, spawner.pos.1 - 5.0, 4.0, &text, sprites);
        }
        WobjType::PlayerSpawn => draw_rect(384, 1216, 32, 32),
        WobjType::FinishTrigger { .. } => draw_rect(352, 2992, 32, 32),
        WobjType::GravityTrigger { .. } => draw_rect(480, 1344, 32, 32),
        WobjType::CustomizeableMoveablePlatform {
            bitmap_x32,
            target_relative,
            speed,
            ..
        } => {
            draw_rect(bitmap_x32.0 as i32 * 32, bitmap_x32.1 as i32 * 32, 32, 32);
            sprites.append(
                &mut Sprite::new_rect(
                    (
                        target_relative.0 + spawner.pos.0,
                        target_relative.1 + spawner.pos.1,
                    ),
                    (
                        target_relative.0 + spawner.pos.0 + 32.0,
                        target_relative.1 + spawner.pos.1 + 32.0,
                    ),
                    3.0,
                    (1.0, 1.0, 0.0, 0.8),
                )
                .to_vec(),
            );
            let dist = (target_relative.0.powi(2) + target_relative.1.powi(2)).sqrt();
            let unclamped_pos = editor_data.time_past.as_secs_f32() * (*speed * 30.0);
            let lin_pos = if (unclamped_pos / dist) as i32 % 2 == 0 {
                (unclamped_pos.rem_euclid(dist)) / dist.max(f32::EPSILON)
            } else {
                (dist - unclamped_pos.rem_euclid(dist)) / dist.max(f32::EPSILON)
            };
            sprites.push(Sprite::new_pure_color(
                (
                    spawner.pos.0 + target_relative.0 * lin_pos,
                    spawner.pos.1 + target_relative.1 * lin_pos,
                    0.0,
                ),
                (32.0, 32.0),
                (1.0, 1.0, 0.0, 0.4),
            ));
        },
        WobjType::SkinUnlock { id } => {
            sprites.push(
                Sprite::new(
                    (spawner.pos.0, spawner.pos.1, 0.0),
                    (32.0, 32.0),
                    (448.0, 7520.0, 32.0, 32.0)
                )
            );
            sprites.push(
                Sprite::new(
                    (spawner.pos.0, spawner.pos.1, 0.0),
                    (32.0, 32.0),
                    match id {
			            0 => (128.0, 1824.0, 32.0, 32.0),
			            1 => (256.0, 1824.0, 32.0, 32.0),
			            2 => (256.0, 1920.0-32.0, 32.0, 32.0),
			            3 => (256.0, 3392.0, 32.0, 32.0),
			            4 => (128.0, 1984.0, 32.0, 32.0),
			            5 => (0.0, 1280.0, 32.0, 32.0),
			            6 => (128.0, 1280.0, 32.0, 32.0),
			            7 => (0.0, 768.0, 32.0, 32.0),
			            8 => (128.0, 768.0, 32.0, 32.0),
			            9 => (5.0, 4616.0, 32.0, 32.0),
                        10 => (0.0, 7776.0, 32.0, 32.0),
                        _ => (0.0, 0.0, 0.0, 0.0),
                    }
                )
            );
        },
        WobjType::CoolPlatform {
            time_off_before,
            time_on,
            time_off_after
        } => {
            let mut sprite = Sprite::new(
                (spawner.pos.0, spawner.pos.1, 0.0),
                (32.0, 32.0),
                (352.0, 2112.0, 32.0, 32.0),
            );
            let time = editor_data
                .time_past
                .as_secs_f32()
                .rem_euclid((*time_off_before as f32 / 30.0) + (*time_on as f32 / 30.0) + (*time_off_after as f32 / 30.0));
            if time > (*time_off_after) as f32 / 30.0 && time < (*time_off_after + *time_on) as f32 / 30.0 {
                sprite.tint = [1.0, 1.0, 1.0, 1.0];
            } else {
                sprite.tint = [0.5, 0.5, 0.5, 0.5];
            }
            sprites.push(sprite);
        },
    }
}
