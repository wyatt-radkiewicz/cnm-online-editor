use cnmo_parse::cnma::{Mode, ResourceId, MaxPowerAbility};
use eframe::egui;

use crate::{editor_data::EditorData, camera::Camera, instanced_sprites::{Sprite, InstancedSprites}};

crate::create_instance_resource!(GfxPreviewResource);

pub struct GameConfigPanel {
    pub selected_mode: Option<usize>,
    pub drag: usize,
    pub drag_source: Option<usize>,
    pub new_level: String,
    pub preview_gfx: bool,
}

impl GameConfigPanel {
    pub fn new() -> Self {
        Self {
            selected_mode: None,
            drag: 0,
            drag_source: None,
            new_level: "".to_string(),
            preview_gfx: false,
        }
    }

    pub fn update(&mut self, ui: &mut egui::Ui, editor_data: &mut EditorData) {
        if self.preview_gfx {
            egui::ScrollArea::new([true, true]).auto_shrink([false, false]).show(ui, |ui| {
                let size = editor_data.gfx_size;
                let (rect, response) = ui.allocate_exact_size(
                    egui::vec2(size.0 as f32 * 2.0, size.1 as f32 * 2.0),
                    egui::Sense::hover()
                );
                let mut camera = Camera::new();
                camera.set_projection(size.0 as f32, size.1 as f32, None, true);
                let sprites = vec![Sprite::new(
                    (0.0, 0.0, 0.0),
                    (size.0 as f32, size.1 as f32),
                    (0.0, 0.0, size.0 as f32, size.1 as f32))];
                InstancedSprites::new()
                    .with_camera(camera)
                    .with_sprites(sprites)
                    .paint::<GfxPreviewResource>(ui, rect);
                if let Some(mut pos) = response.ctx.pointer_latest_pos() {
                    pos.x -= rect.left_top().x;
                    pos.y -= rect.left_top().y;
                    pos.x /= 2.0;
                    pos.y /= 2.0;
                    editor_data.info_bar = format!(
                        "pixel: ({}, {}), snap 32x32: ({}, {}), snap 16x16: ({}, {}), snap 8x8: ({}, {}), grid 32x32: ({}, {}), grid 16x16: ({}, {}), grid 8x8: ({}, {})",
                        pos.x as i32, pos.y as i32,
                        ((pos.x / 32.0).floor() * 32.0) as i32, ((pos.y / 32.0).floor() * 32.0) as i32,
                        ((pos.x / 16.0).floor() * 16.0) as i32, ((pos.y / 16.0).floor() * 16.0) as i32,
                        ((pos.x / 8.0).floor() * 8.0) as i32, ((pos.y / 8.0).floor() * 8.0) as i32,
                        (pos.x / 32.0).floor() as i32, (pos.y / 32.0).floor() as i32,
                        (pos.x / 16.0).floor() as i32, (pos.y / 16.0).floor() as i32,
                        (pos.x / 8.0).floor() as i32, (pos.y / 8.0).floor() as i32,
                    );
                }
            });
            return;
        }

        editor_data.info_bar = "Game config editor".to_string();
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            if self.selected_mode == None {
                ui.label("Click A section to edit its contents!");
                return;
            }
    
            let mode_idx = self.selected_mode.unwrap();
            if ui.button("Delete Mode").clicked() {
                editor_data.game_config_file.modes.remove(mode_idx);
                self.selected_mode = None;
                self.drag = 0;
                self.drag_source = None;
                return;
            }
    
            match &mut editor_data.game_config_file.modes[mode_idx] {
                Mode::MusicIds(resources) |
                Mode::SoundIds(resources) => {
                    let mut delete_idx = None;
                    for (idx, resource) in resources.iter_mut().enumerate() {
                        ui.horizontal(|ui| {
                            ui.add(egui::DragValue::new(&mut resource.id).clamp_range(0..=63)).on_hover_text("The resource ID");
                            ui.text_edit_singleline(&mut resource.path);
                            if ui.button("Remove").clicked() {
                                delete_idx = Some(idx);
                            }
                        });
                    }
                    if let Some(idx) = delete_idx {
                        resources.remove(idx);
                    }
                    if ui.button("Add").clicked() {
                        resources.push(ResourceId { id: resources.len() as u32, path: "".to_string() });
                    }
                },
                Mode::MusicVolumeOverride => {
                    ui.label("This section is unused.");
                },
                Mode::LevelSelectOrder(levels) => {
                    let mut delete_idx = None;
                    for (idx, level) in levels.iter().enumerate() {
                        if self.drag == idx && Some(self.drag) != self.drag_source && self.drag_source != None {
                            ui.label("Move here");
                        }
                        ui.horizontal(|ui| {
                            let response = ui.label(level);
                            if ui.button("Remove").clicked() {
                                delete_idx = Some(idx);
                            }
                            if ui.rect_contains_pointer(response.rect) {
                                if response.ctx.input().pointer.primary_clicked() {
                                    self.drag_source = Some(idx);
                                }
                                self.drag = idx;
                            }
                        });
                    }
            
                    if let Some(idx) = delete_idx {
                        levels.remove(idx);
                        self.drag = 0;
                        self.drag_source = None;
                    }
                    if let Some(src) = self.drag_source {
                        if self.drag != src {
                            egui::Area::new("level_order_dragging")
                                .interactable(false)
                                .fixed_pos(ui.ctx().pointer_interact_pos().unwrap_or_default())
                                .show(ui.ctx(), |ui| {
                                ui.label(levels[src].to_string());
                            });
                        }
                    }
                    if ui.ctx().input().pointer.any_released() {
                        if let Some(src) = self.drag_source {
                            let temp = levels.remove(src);
                            if self.drag <= src {
                                levels.insert(self.drag, temp);
                            } else {
                                levels.insert(self.drag - 1, temp);
                            }
                        }
                        self.drag = 0;
                        self.drag_source = None;
                    }
                    if ui.text_edit_singleline(&mut self.new_level).lost_focus() {
                        levels.push(self.new_level.clone());
                        self.new_level.clear();
                    }
                },
                Mode::MaxPowerDef(def) => {
                    ui.horizontal(|ui| {
                        ui.label("Skin ID: ");
                        ui.add(egui::DragValue::new(&mut def.id).clamp_range(0..=16)).on_hover_text("The skin ID this power is for");
                    });
                    ui.horizontal(|ui| {
                        ui.label("Speed multiplier: ");
                        ui.add(egui::DragValue::new(&mut def.speed).clamp_range(0..=99)).on_hover_text("Speed multiplier");
                    });
                    ui.horizontal(|ui| {
                        ui.label("Jump multiplier: ");
                        ui.add(egui::DragValue::new(&mut def.jump).clamp_range(0..=99)).on_hover_text("Jump multiplier");
                    });
                    ui.horizontal(|ui| {
                        ui.label("Strength multiplier: ");
                        ui.add(egui::DragValue::new(&mut def.strength).clamp_range(0..=999_999)).on_hover_text("Strength multiplier");
                    });
                    ui.horizontal(|ui| {
                        ui.label("Gravity multiplier: ");
                        ui.add(egui::DragValue::new(&mut def.gravity).clamp_range(0..=99)).on_hover_text("The gravity you have with this skin");
                    });
                    ui.horizontal(|ui| {
                        ui.label("Hp Drain (per second): ");
                        ui.add(egui::DragValue::new(&mut def.hpcost).clamp_range(0..=999_999)).on_hover_text("How much HP you lose per second using this ability");
                    });
                    
                    if let Some(ability) = &mut def.ability {
                        egui::ComboBox::new("max_power_combo_box", "Max Power Ability")
                            .selected_text(ability.to_string())
                            .show_ui(ui, |ui| {
                            ui.selectable_value(ability, MaxPowerAbility::DoubleJump, "Double Jump");
                            ui.selectable_value(ability, MaxPowerAbility::Flying, "Wings Flight");
                            ui.selectable_value(ability, MaxPowerAbility::MarioBounce, "Bounce on Enemies");
                            ui.selectable_value(ability, MaxPowerAbility::DropShield, "Temporary Sheild When Landing on Ground");
                        });
                        if ui.button("Don't use ability").clicked() {
                            def.ability = None;
                        }
                    } else {
                        if ui.button("Use ability").clicked() {
                            def.ability = Some(MaxPowerAbility::DoubleJump);
                        }
                    }
                },
                Mode::LuaAutorunCode(code) => {
                    ui.add(
                        egui::TextEdit::multiline(code)
                            .font(egui::TextStyle::Monospace)
                            .code_editor()
                            .lock_focus(true)
                            .desired_width(f32::INFINITY)
                    );
                },
            }
        });
    }
}
