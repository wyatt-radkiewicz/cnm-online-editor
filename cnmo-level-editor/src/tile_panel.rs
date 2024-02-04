use eframe::egui;
use cnmo_parse::lparse::level_data::{
    self,
    LevelData,
    cnmb_types::{
        DamageType, CollisionType,
    },
};
use crate::editor_data::EditorData;
use crate::instanced_sprites::{InstancedSprites, Sprite};
//use crate::common_gfx::GfxCommonResources;
use crate::camera::Camera;

crate::create_instance_resource!(TilePanelSpriteInstances);
crate::create_instance_resource!(TilePanelCollisionDataSpriteInstances);
crate::create_instance_resource!(TilePanelPreviewSpriteInstances);

pub struct TilePanel {
    picking_image: Option<Option<usize>>,
    top_left: Option<(i32, i32)>,
    bottom_right: Option<(i32, i32)>,
    picking_origin: Option<(i32, i32)>,
    scroll_offset: f32,
}

impl TilePanel {
    pub fn new() -> Self {
        Self {
            picking_image: None,
            top_left: None,
            bottom_right: None,
            picking_origin: None,
            scroll_offset: 0.0,
        }
    }

    pub fn update(&mut self, ui: &mut egui::Ui, level_data: &mut LevelData, editor_data: &mut EditorData) {
        editor_data.info_bar = "Tile editor".to_string();
        if editor_data.selected_tiles.len() > 1 || editor_data.selected_tiles.len() == 0 {
            ui.label("Please select only 1 tile to edit!");
            return;
        }
        egui::ScrollArea::new([true, true]).auto_shrink([false, false]).show(ui, |ui| {
            let tile = &mut level_data.tile_properties[editor_data.selected_tiles[0]];
            let size = editor_data.gfx_size;
            if let Some(frame_idx) = self.picking_image {
                let (rect, response) = ui.allocate_exact_size(
                    ui.available_size(),
                    egui::Sense::hover().union(egui::Sense::click_and_drag()),
                );
                let mut camera = Camera::new();
                let cw = size.0 as f32;
                let ch = (rect.height() / rect.width()) * size.0 as f32;
                self.scroll_offset -= response.ctx.input().scroll_delta.y;
                self.scroll_offset = self.scroll_offset.clamp(0.0, (size.1 as f32 - ch).max(0.0));
                camera.set_projection(cw, ch, None, true);
                let mut sprites = vec![Sprite::new(
                    (0.0, 0.0, 0.0),
                    (size.0 as f32, size.1 as f32),
                    (0.0, self.scroll_offset, size.0 as f32, size.1 as f32))];
                if let (Some(top_left), Some(bottom_right)) = (self.top_left, self.bottom_right) {
                    sprites.push(Sprite::new_pure_color(
                        (top_left.0 as f32 * 32.0, top_left.1 as f32 * 32.0 - self.scroll_offset, 0.0),
                        ((bottom_right.0 - top_left.0 + 1) as f32 * 32.0, (bottom_right.1 - top_left.1 + 1) as f32 * 32.0),
                        (1.0, 1.0, 0.0, 0.2),
                    ));
                }
                InstancedSprites::new()
                    .with_camera(camera)
                    .with_sprites(sprites)
                    .paint::<TilePanelSpriteInstances>(ui, rect);
                if response.drag_started() {
                    if let Some(mut pos) = response.interact_pointer_pos() {
                        pos -= egui::vec2(rect.min.x, rect.min.y);
                        pos = egui::pos2(pos.x / rect.width() * cw, pos.y / rect.height() * ch);
                        pos += egui::vec2(0.0, self.scroll_offset);
                        self.picking_origin = Some((pos.x as i32 / 32, pos.y as i32 / 32));
                    }
                }
                if response.dragged() {
                    if let Some(mut pos) = response.interact_pointer_pos() {
                        ui.output().cursor_icon = egui::CursorIcon::Grabbing;
                        pos -= egui::vec2(rect.min.x, rect.min.y);
                        pos = egui::pos2(pos.x / rect.width() * cw, pos.y / rect.height() * ch);
                        pos += egui::vec2(0.0, self.scroll_offset);
                        let pos2 = (pos.x as i32 / 32, pos.y as i32 / 32);
                        if let Some(origin) = self.picking_origin {
                            self.top_left = Some((origin.0.min(pos2.0), origin.1.min(pos2.1)));
                            self.bottom_right = Some((origin.0.max(pos2.0), origin.1.max(pos2.1)));
                        }
                    }
                }
                if response.drag_released() {
                    let mut push_index = if let Some(idx) = frame_idx {
                        tile.frames.remove(idx);
                        idx
                    } else {
                        tile.frames.len()
                    };
                    for y in self.top_left.unwrap().1..=self.bottom_right.unwrap().1 {
                        for x in self.top_left.unwrap().0..=self.bottom_right.unwrap().0 {
                            if tile.frames.len() < level_data.version.get_max_tile_frames() {
                                tile.frames.insert(push_index, (x, y));
                                push_index += 1;
                            }
                        }
                    }

                    self.picking_image = None;
                    self.bottom_right = None;
                    self.top_left = None;
                }
                return;
            }

            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.heading("General Properties");
            });
            ui.separator();
            egui::Grid::new("tile_properties_grid")
                .num_columns(2)
                .striped(true)
                .show(ui, |ui| {
                ui.label("Is: ");
                ui.checkbox(&mut tile.solid, "Solid");
                ui.end_row();
                ui.label("Transparency: ");
                ui.horizontal(|ui| {
                    ui.add(egui::Slider::new(&mut tile.transparency, 0..=level_data::consts::CLEAR));
                    ui.label("(help?)").on_hover_ui_at_pointer(|ui| {
                        ui.label("0 is fully opaque, and 7 (the max) is fully transparent");
                    });
                });
                ui.end_row();
                ui.label("Angle: ");
                ui.add(egui::Slider::new(&mut tile.angle, 0..=359));
                ui.end_row();
                ui.label("Damage type: ");
                egui::ComboBox::new("damage_type_combo_box", "")
                    .selected_text(get_damage_type_name(&tile.damage_type))
                    .show_ui(ui, |ui| {
                    if ui.selectable_label(matches!(tile.damage_type, DamageType::None), get_damage_type_name(&DamageType::None)).clicked() {
                        tile.damage_type = DamageType::None;
                    }
                    if ui.selectable_label(matches!(tile.damage_type, DamageType::Lava(_)), get_damage_type_name(&DamageType::Lava(0))).clicked() {
                        tile.damage_type = DamageType::Lava(0);
                    }
                    if ui.selectable_label(matches!(tile.damage_type, DamageType::Spikes(_)), get_damage_type_name(&DamageType::Spikes(0))).clicked() {
                        tile.damage_type = DamageType::Spikes(1);
                    }
                    if ui.selectable_label(matches!(tile.damage_type, DamageType::Quicksand(_)), get_damage_type_name(&DamageType::Quicksand(0))).clicked() {
                        tile.damage_type = DamageType::Quicksand(1);
                    }
                    if ui.selectable_label(matches!(tile.damage_type, DamageType::Ice(_)), get_damage_type_name(&DamageType::Ice(1.0))).clicked() {
                        tile.damage_type = DamageType::Ice(1.0);
                    }
                    if ui.selectable_label(matches!(tile.damage_type, DamageType::Splashes(_)), get_damage_type_name(&DamageType::Splashes(0))).clicked() {
                        tile.damage_type = DamageType::Splashes(0);
                    }
                });
                ui.end_row();
                match &mut tile.damage_type {
                    DamageType::Lava(dmg) | 
                    DamageType::Spikes(dmg) | 
                    DamageType::Splashes(dmg) |
                    DamageType::Quicksand(dmg) => {
                        ui.label("Damage delt per frame: ");
                        ui.horizontal(|ui| {
                            ui.add(egui::DragValue::new(dmg).speed(1.0));
                            ui.label("(help?)").on_hover_ui_at_pointer(|ui| {
                                ui.label("This is the amount of hp delt to the player per frame.");
                                ui.label("Postive values indicate the player taking damage.");
                                ui.label("Negative values heal the player and 0 will do nothing.");
                                ui.label("A damage type of \"Lava\" and 0 damage will effectivly be water");
                            });
                        });
                        ui.end_row();
                    },
                    DamageType::Ice(ice) => {
                        ui.label("Ice friction multiplier");
                        ui.add(egui::DragValue::new(ice).speed(0.1));
                        ui.end_row();
                    },
                    _ => {},
                }
                ui.label("Animation speed: ");
                ui.horizontal(|ui| {
                    ui.add(egui::DragValue::new(&mut tile.anim_speed.0).clamp_range(1..=i32::MAX).speed(1.0));
                    ui.label("(help?)").on_hover_ui_at_pointer(|ui| {
                        ui.label("This is the amount of frames it takes before the tile switches to the next frame.");
                    });
                });
                ui.end_row();
            });
            ui.separator();
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.heading("Animation Frames");
            });
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Min).with_main_wrap(true), |ui| {
                let mut remove_idx = None;
                let mut sprites = vec![];
                let avail_size = ui.available_size_before_wrap();
                let (rect, response) = ui.allocate_exact_size(
                    egui::vec2(avail_size.x, (tile.frames.len() / (avail_size.x / 33.0).floor() as usize + 1) as f32 * 33.0),
                    egui::Sense::click()
                );
                let mut camera = Camera::new();
                camera.set_projection(rect.width(), rect.height(), None, true);
                let mut idx = 0;
                let items_per_row = (camera.get_proj_size_world_space().x / 33.0) as usize;
                let (mut ix, mut iy) = (0.0, 0.0);
                for frame in tile.frames.iter() {
                    sprites.push(Sprite::new(
                        (ix, iy, 0.0),
                        (32.0, 32.0),
                        (frame.0 as f32 * 32.0, frame.1 as f32 * 32.0, 32.0, 32.0)
                    ));
                    if let Some(mut pos) = response.interact_pointer_pos() {
                        pos -= egui::vec2(rect.min.x, rect.min.y);
                        if pos.x > ix && pos.x < ix + 33.0 && pos.y > iy && pos.y < iy + 33.0 {
                            if response.clicked_by(egui::PointerButton::Primary) {
                                self.picking_image = Some(Some(idx));
                            } else if response.clicked_by(egui::PointerButton::Secondary) {
                                remove_idx = Some(idx);
                            }
                        }
                    }
                    idx += 1;
                    ix += 33.0;
                    if idx % items_per_row == 0 {
                        ix = 0.0;
                        iy += 33.0;
                    }
                }
                InstancedSprites::new()
                        .with_camera(camera)
                        .with_sprites(sprites)
                        .paint::<TilePanelSpriteInstances>(ui, rect);
                if let Some(remove_idx) = remove_idx {
                    if tile.frames.len() > 1 {
                        tile.frames.remove(remove_idx);
                    }
                }
                if tile.frames.len() < level_data.version.get_max_tile_frames() {
                    if ui.button("Add New Frame").clicked() {
                        self.picking_image = Some(None);
                    }
                }
            });
            ui.separator();
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.heading("Collision Data");
            });
            ui.horizontal(|ui| {
                ui.label("Collision type: ");
                egui::ComboBox::new("collision_data_combo_box", "")
                    .selected_text(get_collision_type_name(&tile.collision_data))
                    .show_ui(ui, |ui| {
                    if ui.selectable_label(matches!(tile.collision_data, CollisionType::Box(_)), "Box").clicked() {
                        tile.collision_data = CollisionType::Box(cnmo_parse::Rect { x: 0, y: 0, w: 32, h: 32 });
                    }
                    if ui.selectable_label(matches!(tile.collision_data, CollisionType::Jumpthrough(_)), "Jumpthrough").clicked() {
                        tile.collision_data = CollisionType::Jumpthrough(cnmo_parse::Rect { x: 0, y: 0, w: 32, h: 32 });
                    }
                    if ui.selectable_label(matches!(tile.collision_data, CollisionType::Heightmap(_)), "Heightmap").clicked() {
                        tile.collision_data = CollisionType::Heightmap([32u8; 32]);
                    }
                });
            });
            match &mut tile.collision_data {
                &mut CollisionType::Box(ref mut rect) |
                &mut CollisionType::Jumpthrough(ref mut rect) => {
                    ui.horizontal(|ui| {
                        ui.label(format!("Left: {}", rect.x));
                        ui.label(format!("Right: {}", rect.x+rect.w));
                    });
                    ui.horizontal(|ui| {
                        ui.label(format!("Top: {}", rect.y));
                        ui.label(format!("Bottom: {}", rect.y+rect.h));
                    });
                    let (uirect, response) = ui.allocate_exact_size(egui::Vec2::splat(256.0), egui::Sense::click_and_drag());
                    let camera = Camera::new().with_projection(32.0, 32.0, None, true);
                    let sprites = vec![
                        Sprite::new((0.0, 0.0, 0.0), (32.0, 32.0), (tile.frames[0].0 as f32 * 32.0, tile.frames[0].1 as f32 * 32.0, 32.0, 32.0)),
                        Sprite::new_pure_color((rect.x as f32, rect.y as f32, 0.0), (rect.w as f32, rect.h as f32), (1.0, 0.0, 1.0, 0.3)),
                    ];
                    InstancedSprites::new()
                        .with_camera(camera)
                        .with_sprites(sprites)
                        .paint::<TilePanelCollisionDataSpriteInstances>(ui, uirect);
                    let point = (
                        (((response.interact_pointer_pos().unwrap_or_default().x - uirect.min.x) / 8.0).floor() as i32).clamp(0, 32),
                        (((response.interact_pointer_pos().unwrap_or_default().y - uirect.min.y) / 8.0).floor() as i32).clamp(0, 32)
                    );
                    if response.drag_started() {
                        self.picking_origin = Some(point);
                    }
                    if response.dragged() {
                        if let Some(origin) = self.picking_origin {
                            rect.x = origin.0.min(point.0);
                            rect.y = origin.1.min(point.1);
                            rect.w = origin.0.max(point.0) - rect.x;
                            rect.h = origin.1.max(point.1) - rect.y;
                        }
                    }
                },
                &mut CollisionType::Heightmap(ref mut heightmap) => {
                    let (uirect, response) = ui.allocate_exact_size(egui::Vec2::splat(256.0), egui::Sense::click_and_drag());
                    let camera = Camera::new().with_projection(32.0, 32.0, None, true);
                    let mut sprites = vec![
                        Sprite::new((0.0, 0.0, 0.0), (32.0, 32.0), (tile.frames[0].0 as f32 * 32.0, tile.frames[0].1 as f32 * 32.0, 32.0, 32.0)),
                    ];
                    for (idx, height) in heightmap.iter().enumerate() {
                        sprites.push(Sprite::new_pure_color(
                            (idx as f32, (32 - *height) as f32, 0.0),
                            (1.0, *height as f32),
                            (1.0, 0.0, 1.0, 0.3)
                        ));
                    }
                    InstancedSprites::new()
                        .with_camera(camera)
                        .with_sprites(sprites)
                        .paint::<TilePanelCollisionDataSpriteInstances>(ui, uirect);
                    let point = (
                        (((response.interact_pointer_pos().unwrap_or_default().x - uirect.min.x) / 8.0).floor() as i32).clamp(0, 31),
                        (((response.interact_pointer_pos().unwrap_or_default().y - uirect.min.y) / 8.0).floor() as i32).clamp(0, 32)
                    );
                    if response.dragged() {
                        heightmap[point.0 as usize] = 32u8 - point.1 as u8;
                    }
                },
            };

            egui::Window::new("Tile Preview")
                .fixed_size((96.0, 96.0))
                .collapsible(true)
                .show(ui.ctx(), |ui| {
                let camera = Camera::new().with_projection(32.0, 32.0, None, true);
                let idx = (editor_data.time_past.as_secs_f32() * (30.0 / tile.anim_speed.0 as f32)) as usize % tile.frames.len();
                let mut sprite = Sprite::new(
                    (0.0, 0.0, 0.0),
                    (32.0, 32.0),
                    (tile.frames[idx].0 as f32 * 32.0, tile.frames[idx].1 as f32 * 32.0, 32.0, 32.0)
                );
                sprite.tint = [1.0, 1.0, 1.0, 1.0 - (tile.transparency as f32 / (level_data::consts::CLEAR as f32 - f32::EPSILON))];
                let (rect, _response) = ui.allocate_at_least(egui::vec2(96.0, 96.0), egui::Sense::focusable_noninteractive());
                InstancedSprites::new()
                    .with_camera(camera)
                    .with_sprites(vec![sprite])
                    .paint::<TilePanelPreviewSpriteInstances>(ui, rect);
            });

            ui.separator();
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.heading("Delete Tile");
            });
            if ui.button("Delete this tile").clicked() {
                // First delete this tile from the level cells
                let tile_id = editor_data.selected_tiles[0] as u16;
                for cell in level_data.cells.cells_mut().iter_mut() {
                    if cell.foreground.0 == Some(tile_id) {
                        cell.foreground.0 = None;
                    }
                    if cell.background.0 == Some(tile_id) {
                        cell.background.0 = None;
                    }
                    if let Some(foreground) = &mut cell.foreground.0 {
                        if *foreground > tile_id {
                            *foreground -= 1;
                        }
                    }
                    if let Some(background) = &mut cell.background.0 {
                        if *background > tile_id {
                            *background -= 1;
                        }
                    }
                }

                // Delete the tile
                level_data.tile_properties.remove(tile_id as usize);

                // Clear selections of tiles
                editor_data.reset_selected_tiles();
            }
        });
    }
}

fn get_damage_type_name(dmg_type: &DamageType) -> &str {
    match dmg_type {
        DamageType::None => "None",
        DamageType::Lava(..) => "Lava",
        DamageType::Quicksand(..) => "Quicksand",
        DamageType::Spikes(..) => "Spikes",
        DamageType::Ice(..) => "Ice",
        DamageType::Splashes(..) => "Splashes",
    }
}

fn get_collision_type_name(collision_type: &CollisionType) -> &str {
    match collision_type {
        CollisionType::Box(..) => "Box",
        CollisionType::Jumpthrough(..) => "Jumpthrough",
        CollisionType::Heightmap(..) => "Heightmap",
    }
}
