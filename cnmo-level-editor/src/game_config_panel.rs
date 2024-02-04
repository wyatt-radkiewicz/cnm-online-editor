use cnmo_parse::cnma::{MaxPowerAbility, Mode, ResourceId, PetAI};
use eframe::egui;

use crate::{
    camera::Camera,
    editor_data::EditorData,
    instanced_sprites::{InstancedSprites, Sprite},
};

crate::create_instance_resource!(GfxPreviewResource);

pub struct GameConfigPanel {
    pub selected_mode: Option<usize>,
    pub drag: usize,
    pub drag_source: Option<usize>,
    pub new_level: String,
    pub preview_gfx: bool,
    preview_scroll_offset: f32,
}

impl GameConfigPanel {
    pub fn new() -> Self {
        Self {
            selected_mode: None,
            drag: 0,
            drag_source: None,
            new_level: "".to_string(),
            preview_gfx: false,
            preview_scroll_offset: 0.0,
        }
    }

    pub fn update(&mut self, ui: &mut egui::Ui, editor_data: &mut EditorData) {
        if self.preview_gfx {
            egui::ScrollArea::new([true, true]).auto_shrink([false, false]).show(ui, |ui| {
                //let size = editor_data.gfx_size;
                let (rect, response) = ui.allocate_exact_size(
                    ui.available_size(),
                    //egui::vec2(size.0 as f32 * 2.0, size.1 as f32 * 2.0),
                    egui::Sense::hover()
                );
                let cw = editor_data.gfx_size.0 as f32;
                let ch = (rect.height() / rect.width()) * editor_data.gfx_size.0 as f32;
                self.preview_scroll_offset -= response.ctx.input().scroll_delta.y;
                self.preview_scroll_offset = self
                    .preview_scroll_offset
                    .clamp(0.0, editor_data.gfx_size.1 as f32 - ch);
                let mut camera = Camera::new();
                camera.set_projection(cw, ch, None, true);
                let sprites = vec![Sprite::new(
                    (0.0, 0.0, 0.0),
                    (editor_data.gfx_size.0 as f32, editor_data.gfx_size.1 as f32),
                    (0.0, self.preview_scroll_offset, editor_data.gfx_size.0 as f32, editor_data.gfx_size.1 as f32))];
                InstancedSprites::new()
                    .with_camera(camera)
                    .with_sprites(sprites)
                    .paint::<GfxPreviewResource>(ui, rect);
                if let Some(mut pos) = response.ctx.pointer_latest_pos() {
                    pos.x -= rect.left_top().x;
                    pos.y -= rect.left_top().y;
                    pos.x = pos.x / rect.width() * cw;
                    pos.y = pos.y / rect.height() * ch + self.preview_scroll_offset;
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
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
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
                    Mode::MusicIds(resources) | Mode::SoundIds(resources) => {
                        let mut delete_idx = None;
                        for (idx, resource) in resources.iter_mut().enumerate() {
                            ui.horizontal(|ui| {
                                ui.add(egui::DragValue::new(&mut resource.id).clamp_range(0..=255))
                                    .on_hover_text("The resource ID");
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
                            resources.push(ResourceId {
                                id: resources.len() as u32,
                                path: "".to_string(),
                            });
                        }
                    }
                    Mode::MusicVolumeOverride => {
                        ui.label("This section is unused.");
                    }
                    Mode::LevelSelectOrder(ref mut levels) => {
                        let mut delete_idx = None;
                        for (idx, level) in levels.iter_mut().enumerate() {
                            if self.drag == idx
                                && Some(self.drag) != self.drag_source
                                && self.drag_source != None
                            {
                                ui.label("Move here");
                            }
                            ui.horizontal(|ui| {
                                let response = ui.label(level.0.to_string());
                                ui.add(egui::DragValue::new(&mut level.1));
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
                                        ui.label(levels[src].0.to_string());
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
                            levels.push((self.new_level.clone(), 0));
                            self.new_level.clear();
                        }
                    }
                    Mode::MaxPowerDef(def) => {
                        ui.horizontal(|ui| {
                            ui.label("Skin ID: ");
                            ui.add(egui::DragValue::new(&mut def.id).clamp_range(0..=16))
                                .on_hover_text("The skin ID this power is for");
                        });
                        ui.horizontal(|ui| {
                            ui.label("Speed multiplier: ");
                            ui.add(egui::DragValue::new(&mut def.speed).clamp_range(0..=99))
                                .on_hover_text("Speed multiplier");
                        });
                        ui.horizontal(|ui| {
                            ui.label("Jump multiplier: ");
                            ui.add(egui::DragValue::new(&mut def.jump).clamp_range(0..=99))
                                .on_hover_text("Jump multiplier");
                        });
                        ui.horizontal(|ui| {
                            ui.label("Strength multiplier: ");
                            ui.add(
                                egui::DragValue::new(&mut def.strength).clamp_range(0..=999_999),
                            )
                            .on_hover_text("Strength multiplier");
                        });
                        ui.horizontal(|ui| {
                            ui.label("Gravity multiplier: ");
                            ui.add(egui::DragValue::new(&mut def.gravity).clamp_range(0..=99))
                                .on_hover_text("The gravity you have with this skin");
                        });
                        ui.horizontal(|ui| {
                            ui.label("Hp Drain (per second): ");
                            ui.add(egui::DragValue::new(&mut def.hpcost).clamp_range(0..=999_999))
                                .on_hover_text(
                                    "How much HP you lose per second using this ability",
                                );
                        });

                        if let Some(ability) = &mut def.ability {
                            egui::ComboBox::new("max_power_combo_box", "Max Power Ability")
                                .selected_text(get_ability_name(ability))
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        ability,
                                        MaxPowerAbility::DoubleJump,
                                        get_ability_name(&MaxPowerAbility::DoubleJump),
                                    );
                                    ui.selectable_value(
                                        ability,
                                        MaxPowerAbility::Flying,
                                        get_ability_name(&MaxPowerAbility::Flying),
                                    );
                                    ui.selectable_value(
                                        ability,
                                        MaxPowerAbility::MarioBounce,
                                        get_ability_name(&MaxPowerAbility::MarioBounce),
                                    );
                                    ui.selectable_value(
                                        ability,
                                        MaxPowerAbility::DropShield,
                                        get_ability_name(&MaxPowerAbility::DropShield),
                                    );
                                });
                            if ui.button("Don't use ability").clicked() {
                                def.ability = None;
                            }
                        } else {
                            if ui.button("Use ability").clicked() {
                                def.ability = Some(MaxPowerAbility::DoubleJump);
                            }
                        }
                    }
                    Mode::LuaAutorunCode(code) => {
                        ui.add(
                            egui::TextEdit::multiline(code)
                                .font(egui::TextStyle::Monospace)
                                .code_editor()
                                .lock_focus(true)
                                .desired_width(f32::INFINITY),
                        );
                    }
                    Mode::PetDefs(ref mut defs) => {
                        let mut ui_id = 0;
                        for ref mut def in defs.iter_mut() {
                            egui::Grid::new("defs".to_string() + &ui_id.to_string()).num_columns(2).show(ui, |ui| {
                                ui.label("Name: ");
                                ui.add(egui::TextEdit::singleline(&mut def.name));
                                ui.end_row();
                                ui.label("Animation Base X: ").on_hover_text("This is what tile (32x32) in the graphics file it is");
                                ui.add(egui::DragValue::new(&mut def.animbase.0).clamp_range(0..=15));
                                ui.end_row();
                                ui.label("Animation Base Y: ").on_hover_text("This is what tile (32x32) in the graphics file it is");
                                ui.add(egui::DragValue::new(&mut def.animbase.1).clamp_range(0..=1023));
                                ui.end_row();
                                ui.label("Icon Base X: ").on_hover_text("This is what tile (32x32) in the graphics file it is");
                                ui.add(egui::DragValue::new(&mut def.iconbase.0).clamp_range(0..=15));
                                ui.end_row();
                                ui.label("Icon Base Y: ").on_hover_text("This is what tile (32x32) in the graphics file it is");
                                ui.add(egui::DragValue::new(&mut def.iconbase.1).clamp_range(0..=1023));
                                ui.end_row();
                                ui.label("Idle SFX ID: ").on_hover_text("-1 means no sound");
                                ui.add(egui::DragValue::new(&mut def.idle_snd).clamp_range(-1..=255));
                                ui.end_row();

                                ui.label("Pet AI Type: ");
                                egui::ComboBox::new("pet_ai_combo_box".to_string() + &ui_id.to_string(), "")
                                    .selected_text(match def.ai {
                                        PetAI::Fly { .. } => "Fly",
                                        PetAI::Bounce { .. } => "Bounce",
                                        PetAI::Walk { .. } => "Walk",
                                    })
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(&mut def.ai, PetAI::Fly {
                                            num_fly_frames: 1,
                                        }, "Fly");
                                        ui.selectable_value(&mut def.ai, PetAI::Walk {
                                            num_idle_frames: 1,
                                            num_walk_frames: 1,
                                            num_fall_frames: 1,
                                        }, "Walk");
                                        ui.selectable_value(&mut def.ai, PetAI::Bounce {
                                            num_idle_frames: 1,
                                            num_bounce_frames: 1,
                                            bounce_idly: false,
                                            jump_height: 7.0,
                                        }, "Bounce");
                                    });
                                ui.end_row();

                                match &mut def.ai {
                                    &mut PetAI::Fly {
                                        ref mut num_fly_frames,
                                    } => {
                                        ui.label("Number of Flying Frames: ");
                                        ui.add(egui::DragValue::new(num_fly_frames).clamp_range(0..=15));
                                        ui.end_row();
                                    },
                                    &mut PetAI::Bounce {
                                        ref mut num_idle_frames,
                                        ref mut num_bounce_frames,
                                        ref mut bounce_idly,
                                        ref mut jump_height,
                                    } => {
                                        ui.label("Number of Idle Frames: ");
                                        ui.add(egui::DragValue::new(num_idle_frames).clamp_range(0..=15));
                                        ui.end_row();
                                        ui.label("Number of Bouncing Frames: ");
                                        ui.add(egui::DragValue::new(num_bounce_frames).clamp_range(0..=15));
                                        ui.end_row();
                                        ui.label("");
                                        if ui.selectable_label(*bounce_idly, "Bounce When Idle").clicked() {
                                            *bounce_idly = !*bounce_idly;
                                        }
                                        ui.end_row();
                                        ui.label("Jump Thrust: ").on_hover_text("Refrence: player jump thrust is 10");
                                        ui.add(egui::DragValue::new(jump_height).clamp_range(-15.0..=15.0));
                                        ui.end_row();
                                    },
                                    &mut PetAI::Walk {
                                        ref mut num_idle_frames,
                                        ref mut num_walk_frames,
                                        ref mut num_fall_frames,
                                    } => {
                                        ui.label("Number of Idle Frames: ");
                                        ui.add(egui::DragValue::new(num_idle_frames).clamp_range(0..=15));
                                        ui.end_row();
                                        ui.label("Number of Walk Frames: ");
                                        ui.add(egui::DragValue::new(num_walk_frames).clamp_range(0..=15));
                                        ui.end_row();
                                        ui.label("Number of Fall Frames: ");
                                        ui.add(egui::DragValue::new(num_fall_frames).clamp_range(0..=15));
                                        ui.end_row();
                                    },
                                }
                            });
                            ui.separator();
                            ui_id += 1;
                        }
                        ui.horizontal(|ui| {
                            if ui.button("Add Definition").clicked() {
                                defs.push(Default::default());
                            }
                            if ui.button("Remove Definition").clicked() {
                                defs.pop();
                            }
                        });
                    }
                }
            });
    }
}

fn get_ability_name(ability: &MaxPowerAbility) -> &str {
    match ability {
        &MaxPowerAbility::DoubleJump => "Double Jump",
        &MaxPowerAbility::DropShield => "Drop Sheild",
        &MaxPowerAbility::Flying => "Flying",
        &MaxPowerAbility::MarioBounce => "Mario Enemy Bounce",
    }
}
