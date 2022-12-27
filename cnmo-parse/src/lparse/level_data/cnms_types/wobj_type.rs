use crate::lparse::{
    Error,
    LParse,
};

use super::{
    super::Point,
    super::VersionSpecs,
    super::consts::*,
    item_type::ItemType,
};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy)]
pub enum TunesTriggerSize {
    Small,
    Big,
    VeryBig,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy)]
pub enum RuneType {
    Fire,
    Ice,
    Air,
    Lightning,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy)]
pub enum TtNodeType {
    NormalTrigger,
    ChaseTrigger,
    Waypoint(i32),
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy)]
pub enum PushZoneType {
    Horizontal,
    Vertical,
    HorizontalSmall,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy)]
pub enum KeyColor {
    Red,
    Green,
    Blue
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy)]
pub enum RockGuyType {
    Medium,
    Small1,
    Small2 {
        face_left: bool
    },
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy)]
pub enum UpgradeTriggerType {
    Wings,
    DeephausBoots,
    CrystalWings,
    Vortex,
    MaxPowerRune {
        skin_power_override: Option<u8>,
    },
    None,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct Teleport {
    pub name: String,
    pub cost: i32,
    pub loc: Point,
}

impl Teleport {
    fn from_lparse(cnms: &LParse, version: &VersionSpecs, index: usize) -> Result<Self, Error> {
        let start = index * version.teleport_name_size;
        let end = start + version.teleport_name_size;
        let name = match String::from_utf8(
        cnms.try_get_entry("TI_NAME")?
            .try_get_u8()?[start..end]
            .iter()
            .cloned()
            .collect()
        ) {
            Ok(s) => s.trim_end_matches('\0').to_string(),
            Err(_) => return Err(Error::Corrupted(format!("Corrupted teleport entry name. Teleport ID: {index}"))),
        };

        let loc = &cnms
            .try_get_entry("TI_POS")?
            .try_get_f32()?[index * 2..index * 2 + 2];

        Ok(Teleport {
            name,
            cost: cnms
                .try_get_entry("TI_COST")?
                .try_get_i32()?[index],
            loc: Point(loc[0], loc[1]),
        })
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy)]
pub enum BackgroundSwitcherShape {
    Small,
    Horizontal,
    Vertical,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub enum WobjType {
    Teleport(Teleport),
    Slime {
        flying: bool,
    },
    TunesTrigger {
        size: TunesTriggerSize,
        music_id: u32,
    },
    PlayerSpawn,
    TextSpawner {
        dialoge_box: bool,
        text: String,
    },
    MovingPlatform {
        vertical: bool,
        dist: f32,
        speed: f32,
    },
    BreakableWall {
        skin_id: Option<u8>,
        health: f32,
    },
    BackgroundSwitcher {
        shape: BackgroundSwitcherShape,
        enabled_layers: std::ops::Range<u32>,
    },
    DroppedItem {
        item: ItemType,
    },
    WandRune {
        rune_type: RuneType,
    },
    Heavy {
        speed: f32,
        face_left: bool,
    },
    Dragon {
        space_skin: bool,
    },
    BozoPin {
        flying_speed: f32,
    },
    Bozo {
        mark_ii: bool,
    },
    SilverSlime,
    LavaMonster {
        face_left: bool,
    },
    TtMinion {
        small: bool,
    },
    SlimeWalker,
    MegaFish {
        water_level: i32,
        swimming_speed: f32,
    },
    LavaDragonHead {
        // Can only be 32 long
        len: u32,
        health: f32,
    },
    TtNode {
        node_type: TtNodeType,
    },
    TtBoss {
        speed: f32,
    },
    EaterBug {
        pop_up_speed: f32,
    },
    SpiderWalker {
        speed: f32,
    },
    SpikeTrap,
    RotatingFireColunmPiece {
        // Pieces should be put together horizontally in the original editor,
        // And this only specifies the x axis of the origin point of the rotating fire column
        origin_x: i32,
        degrees_per_second: f32,
    },
    MovingFire {
        vertical: bool,
        dist: i32,
        speed: f32,
    },
    SuperDragon {
        // Can only have up to 16 waypoint ids
        waypoint_id: u8,
    },
    SuperDragonLandingZone {
        // Can only have up to 16 waypoints
        waypoint_id: u8,
    },
    BozoLaserMinion {
        speed: f32,
    },
    Checkpoint,
    SpikeGuy,
    BanditGuy {
        speed: f32,
    },
    PushZone {
        push_zone_type: PushZoneType,
        push_speed: f32,
    },
    VerticalWindZone {
        acceleration: f32,
    },
    DisapearingPlatform {
        time_on: f32,
        time_off: f32,
    },
    KamakaziSlime,
    SpringBoard {
        jump_velocity: f32,
    },
    Jumpthrough {
        big: bool,
    },
    BreakablePlatform {
        time_till_fall: f32,
    },
    LockedBlock {
        color: KeyColor,
        consume_key: bool,
    },
    RockGuy {
        rock_guy_type: RockGuyType,
    },
    RockGuySlider,
    RockGuySmasher,
    HealthSetTrigger {
        target_health: f32,
    },
    Vortex {
        attract_enemies: bool,
    },
    CustomizeableMoveablePlatform {
        bitmap_x32: (u32, u32),
        target_relative: Point,
        speed: f32,
        start_paused: bool,
        one_way: bool,
    },
    GraphicsChangeTrigger {
        gfx_file: String,
    },
    BossBarInfo {
        boss_name: String,
    },
    BgSpeed {
        vertical_axis: bool,
        layer: u32,
        speed: f32,
    },
    BgTransparency {
        layer: u32,
        transparency: u8,
    },
    TeleportTrigger1 {
        link_id: u32,
        delay_secs: f32,
    },
    TeleportArea1 {
        link_id: u32,
        loc: Point,
    },
    SfxPoint {
        sound_id: u32,
    },
    Wolf,
    Supervirus,
    Lua {
        lua_wobj_type: u8,
    },
    UpgradeTrigger {
        trigger_type: UpgradeTriggerType,
    },
}

impl WobjType {
    pub(super) fn from_lparse(cnms: &LParse, version: &VersionSpecs, index: usize, ignore_warnings: bool) -> Result<Self, Error> {
        let wobj_type_id = cnms.try_get_entry("SP_TYPE")?.try_get_i32()?[index];
        let custom_int = cnms.try_get_entry("SP_CI")?.try_get_i32()?[index];
        let custom_float = cnms.try_get_entry("SP_CF")?.try_get_f32()?[index];

        match wobj_type_id {
            1 => Ok(Self::Teleport(Teleport::from_lparse(cnms, version, custom_int as usize)?)),
            120 => {
                let loc = Teleport::from_lparse(cnms, version, custom_int as usize)?.loc;
                Ok(Self::TeleportArea1 { link_id: custom_float as u32, loc })
            },
            6 => Ok(Self::TunesTrigger { size: TunesTriggerSize::Small, music_id: custom_int as u32 }),
            7 => Ok(Self::TunesTrigger { size: TunesTriggerSize::Big, music_id: custom_int as u32 }),
            89 => Ok(Self::TunesTrigger { size: TunesTriggerSize::VeryBig, music_id: custom_int as u32 }),
            8 => Ok(Self::PlayerSpawn),
            9 | 108 => { // Ending Text/Dialoge Box
                let mut text = "".to_string();
                for i in custom_int as usize..custom_float as usize {
                    text += (super::get_ending_text_line(cnms, version, i).unwrap_or_default() + "\n").as_str();
                }

                Ok(Self::TextSpawner {
                    dialoge_box: wobj_type_id == 108,
                    text
                })
            },
            12 | 87 | 88 => Ok(Self::BackgroundSwitcher {
                shape: match wobj_type_id {
                    12 => BackgroundSwitcherShape::Small,
                    87 => BackgroundSwitcherShape::Horizontal,
                    88 => BackgroundSwitcherShape::Vertical,
                    _ => panic!("Unknown background switcher type!"),
                },
                enabled_layers: custom_int as u32..custom_float as u32,
            }),
            16 => Ok(Self::DroppedItem {
                item: match ItemType::from_item_id(custom_int as u32) {
                    Some(item) => item,
                    None if !ignore_warnings => return Err(Error::Corrupted(format!("Corrupt item ID {custom_int}!"))),
                    None => ItemType::Apple,
                },
            }),
            14 => Ok(Self::DroppedItem { item: ItemType::Knife }),
            15 => Ok(Self::DroppedItem { item: ItemType::Apple }),
            4 => Ok(Self::DroppedItem { item: ItemType::Shotgun }),
            51 | 52 | 53 => Ok(Self::TtNode {
                node_type: match wobj_type_id {
                    51 => TtNodeType::ChaseTrigger,
                    52 => TtNodeType::NormalTrigger,
                    53 => TtNodeType::Waypoint(custom_int),
                    _ => panic!("Unknown TT Node type!"),
                }
            }),
            60 => Ok(Self::RotatingFireColunmPiece {
                origin_x: custom_int,
                degrees_per_second: custom_float * FRAME_RATE as f32,
            }),
            61 | 62 => Ok(Self::MovingFire {
                vertical: wobj_type_id == 61,
                dist: custom_int,
                speed: custom_float,
            }),
            77 | 78 | 80 => Ok(Self::PushZone {
                push_zone_type: match wobj_type_id {
                    77 => PushZoneType::Horizontal,
                    78 => PushZoneType::Vertical,
                    80 => PushZoneType::HorizontalSmall,
                    _ => panic!("Unkown push zone type!"),
                },
                push_speed: custom_float,
            }),
            79 => Ok(Self::VerticalWindZone { acceleration: custom_float }),
            64 => Ok(Self::SuperDragonLandingZone { waypoint_id: custom_int as u8, }),
            90 | 91 => Ok(Self::Jumpthrough { big: wobj_type_id == 91 }),
            104 => Ok(Self::HealthSetTrigger { target_health: custom_float }),
            109 => Ok(Self::GraphicsChangeTrigger { gfx_file: super::get_ending_text_line(cnms, version, custom_int as usize)? }),
            115 => Ok(Self::BossBarInfo { boss_name: super::get_ending_text_line(cnms, version, custom_int as usize)? }),
            116 | 117 => Ok(Self::BgSpeed {
                vertical_axis: wobj_type_id == 117,
                layer: custom_int as u32,
                speed: custom_float,
            }),
            118 => Ok(Self::BgTransparency {
                layer: custom_int as u32,
                transparency: custom_float as u8,
            }),
            119 => Ok(Self::TeleportTrigger1 {
                link_id: custom_int as u32,
                delay_secs: custom_float,
            }),
            121 => Ok(Self::SfxPoint { sound_id: custom_int as u32 }),
            11 | 93 => Ok(Self::BreakableWall {
                skin_id: if wobj_type_id == 11 {
                    None
                } else {
                    Some(custom_int as u8)
                },
                health: custom_float,
            }),
            10 | 82 => Ok(Self::MovingPlatform {
                vertical: wobj_type_id == 82,
                dist: custom_float * custom_int as f32,
                speed: custom_float,
            }),
            83 => Ok(Self::DisapearingPlatform {
                time_on: (custom_int / FRAME_RATE) as f32,
                time_off: custom_float / FRAME_RATE as f32,
            }),
            86 => Ok(Self::SpringBoard { jump_velocity: custom_float }),
            92 => Ok(Self::BreakablePlatform { time_till_fall: custom_int as f32 / FRAME_RATE as f32 }),
            107 => { // Customizable moving platform
                let masked = custom_int as u32 >> 16 & 0xff;
                let vel_x = ((masked >> 2 & 0x3f) as i32 - 32) as f32 + 0.25 * (masked & 0x3) as f32;
                let masked = custom_int as u32 >> 24 & 0xff;
                let vel_y = ((masked >> 2 & 0x3f) as i32 - 32) as f32 + 0.25 * (masked & 0x3) as f32;
                let bitmap_x = custom_int & 0xf;
                let bitmap_y = custom_int >> 4 & 0xfff;

                Ok(Self::CustomizeableMoveablePlatform {
                    bitmap_x32: (bitmap_x as u32, bitmap_y as u32),
                    target_relative: Point(vel_x * custom_float, vel_y * custom_float),
                    speed: (vel_x.powi(2) + vel_y.powi(2)).sqrt(),
                    start_paused: custom_float < 0.0,
                    one_way: custom_float.fract().abs() > f32::EPSILON,
                })
            },
            94 | 95 | 96 => Ok(Self::LockedBlock {
                color: match wobj_type_id {
                    94 => KeyColor::Red,
                    95 => KeyColor::Green,
                    96 => KeyColor::Blue,
                    _ => panic!("Unknown locked block key color!"),
                },
                consume_key: custom_int != 0,
            }),
            105 => Ok(Self::Vortex { attract_enemies: custom_int != 0 }),
            19 | 20 | 21 | 22 => Ok(Self::WandRune {
                rune_type: match wobj_type_id {
                    19 => RuneType::Ice,
                    20 => RuneType::Air,
                    21 => RuneType::Fire,
                    22 => RuneType::Lightning,
                    _ => panic!("Unknown wand rune type!"),
                },
            }),
            114 | 32 | 33 | 34 | 106 | 140 => Ok(Self::UpgradeTrigger {
                trigger_type: match wobj_type_id {
                    114 => UpgradeTriggerType::MaxPowerRune {
                        skin_power_override: if custom_int != 0 {
                            Some(custom_int as u8 - 1u8)
                        } else {
                            None
                        }
                    },
                    32 => UpgradeTriggerType::DeephausBoots,
                    33 => UpgradeTriggerType::Wings,
                    34 => UpgradeTriggerType::CrystalWings,
                    106 => UpgradeTriggerType::Vortex,
                    140 => UpgradeTriggerType::None,
                    _ => panic!("Unknown upgrade trigger type!"),
                },
            }),
            73 => Ok(Self::Checkpoint),
            2 | 35 => Ok(Self::Slime { flying: wobj_type_id == 35 }),
            36 => Ok(Self::Heavy {
                speed: custom_float.abs(),
                face_left: custom_float < 0.0,
            }),
            38 => Ok(Self::Dragon { space_skin: custom_int != 0 }),
            40 => Ok(Self::BozoPin { flying_speed: custom_float }),
            41 | 72 => Ok(Self::Bozo { mark_ii: wobj_type_id == 72 }),
            42 => Ok(Self::SilverSlime),
            43 => Ok(Self::LavaMonster { face_left: custom_int != 0 }),
            44 | 45 => Ok(Self::TtMinion { small: custom_int == 44 }),
            46 => Ok(Self::SlimeWalker),
            47 => Ok(Self::MegaFish {
                water_level: custom_int,
                swimming_speed: custom_float,
            }),
            48 => Ok(Self::LavaDragonHead {
                len: if custom_int > 32 && !ignore_warnings {
                    return Err(Error::Corrupted("Lava dragon boss has too much segments!".to_string()))
                } else {
                    custom_int as u32
                },
                health: custom_float,
            }),
            54 => Ok(Self::TtBoss { speed: custom_float }),
            55 => Ok(Self::EaterBug { pop_up_speed: custom_float }),
            57 => Ok(Self::SpiderWalker { speed: custom_float }),
            59 => Ok(Self::SpikeTrap),
            63 => Ok(Self::SuperDragon { waypoint_id: custom_int as u8 }),
            69 => Ok(Self::BozoLaserMinion { speed: custom_float }),
            74 => Ok(Self::SpikeGuy),
            76 => Ok(Self::BanditGuy { speed: custom_float }),
            84 => Ok(Self::KamakaziSlime),
            97 | 98 | 99 => Ok(Self::RockGuy {
                rock_guy_type: match wobj_type_id {
                    97 => RockGuyType::Medium,
                    98 => RockGuyType::Small1,
                    99 => RockGuyType::Small2 { face_left: custom_int != 0 },
                    _ => panic!("Unknown rock guy type!"),
                },
            }),
            100 => Ok(Self::RockGuySlider),
            101 => Ok(Self::RockGuySmasher),
            122 => Ok(Self::Wolf),
            123 => Ok(Self::Supervirus),
            _ if wobj_type_id >= 124 && wobj_type_id <= 139 => Ok(Self::Lua { lua_wobj_type: (wobj_type_id - 124) as u8 }),
            _ if !ignore_warnings => Err(Error::Corrupted(format!("Unknown wobj type {wobj_type_id}"))),
            _ => Ok(WobjType::Slime { flying: false }),
        }
    }

    pub(super) fn serialize(
        &self,
        teleports: &mut Vec<Teleport>,
        spawns: &mut Vec<f32>,
        checkpoints: &mut Vec<f32>,
        ending_text: &mut Vec<String>,
        spawner: &super::Spawner,
        version: &VersionSpecs,
    ) -> (i32, i32, f32) {
        match self {
            &Self::Teleport(ref teleport) => {
                teleports.push(teleport.clone());
                (1, teleports.len() as i32 - 1, 0.0)
            },
            &Self::TeleportArea1{ link_id, loc } => {
                teleports.push(Teleport { name: "_TELEAREA".to_string(), cost: 0, loc });
                (120, teleports.len() as i32 - 1, link_id as f32)
            },
            &Self::TunesTrigger { size, music_id } => {
                let wobj_type_id = match size {
                    TunesTriggerSize::Small => 6,
                    TunesTriggerSize::Big => 7,
                    TunesTriggerSize::VeryBig => 89,
                };
                (wobj_type_id, music_id as i32, 0.0)
            },
            &Self::PlayerSpawn => {
                spawns.push(spawner.pos.0);
                spawns.push(spawner.pos.1);
                (8, spawns.len() as i32 / 2 - 1, 0.0)
            },
            &Self::TextSpawner { dialoge_box, ref text } => {
                let wobj_type_id = if dialoge_box { 108 } else { 9 };
                if ending_text.len() == version.title_ending_text_line {
                    ending_text.push("".to_string()); // Can't use the line that is used for the title!
                }
                ending_text.push(text.clone());

                (wobj_type_id, ending_text.len() as i32 - 1, 0.0)
            },
            &Self::BackgroundSwitcher { shape, ref enabled_layers } => {
                let wobj_type_id = match shape {
                    BackgroundSwitcherShape::Small => 12,
                    BackgroundSwitcherShape::Horizontal => 87,
                    BackgroundSwitcherShape::Vertical => 88,
                };
                (wobj_type_id, enabled_layers.start as i32, enabled_layers.end as f32)
            },
            &Self::DroppedItem { item } => {
                (16, item.get_item_id() as i32, 0.0)
            },
            &Self::TtNode { node_type } => {
                let (wobj_type_id, custom_int) = match node_type {
                    TtNodeType::ChaseTrigger => (51, 0),
                    TtNodeType::NormalTrigger => (52, 0),
                    TtNodeType::Waypoint(waypoint_id) => (53, waypoint_id),
                };
                (wobj_type_id, custom_int, 0.0)
            },
            &Self::RotatingFireColunmPiece { origin_x, degrees_per_second } => {
                (60, origin_x, degrees_per_second / FRAME_RATE as f32)
            },
            &Self::MovingFire { vertical, dist, speed } => {
                let wobj_type_id = if vertical { 61 } else { 62 };
                (wobj_type_id, dist, speed)
            },
            &Self::PushZone { push_zone_type, push_speed } => {
                let wobj_type_id = match push_zone_type {
                    PushZoneType::Horizontal => 77,
                    PushZoneType::Vertical => 78,
                    PushZoneType::HorizontalSmall => 80,
                };
                (wobj_type_id, 0, push_speed)
            },
            &Self::VerticalWindZone { acceleration } => {
                (79, 0, acceleration)
            },
            &Self::SuperDragonLandingZone { waypoint_id } => {
                (64, waypoint_id as i32, 0.0)
            },
            &Self::Jumpthrough { big } => {
                (if big { 91 } else { 90 }, 0, 0.0)
            },
            &Self::HealthSetTrigger { target_health } => {
                (104, 0, target_health)
            },
            &Self::GraphicsChangeTrigger { ref gfx_file } => {
                if ending_text.len() == version.title_ending_text_line {
                    ending_text.push("".to_string());
                }
                ending_text.push(gfx_file.clone());
                (109, ending_text.len() as i32 - 1, 0.0)
            },
            &Self::BossBarInfo { ref boss_name } => {
                if ending_text.len() == version.title_ending_text_line {
                    ending_text.push("".to_string());
                }
                ending_text.push(boss_name.clone());
                (115, ending_text.len() as i32 - 1, 0.0)
            },
            &Self::BgSpeed { vertical_axis, layer, speed } => {
                (if vertical_axis { 117 } else { 116 }, layer as i32, speed)
            },
            &Self::BgTransparency { layer, transparency } => {
                (118, layer as i32, transparency as f32)
            },
            &Self::TeleportTrigger1 { link_id, delay_secs } => {
                (119, link_id as i32, delay_secs)
            },
            &Self::SfxPoint { sound_id } => {
                (121, sound_id as i32, 0.0)
            },
            &Self::BreakableWall { skin_id, health } => {
                let (wobj_type_id, custom_int) = match skin_id {
                    Some(skin_id) => (93, skin_id as i32),
                    None => (11, 0),
                };
                (wobj_type_id, custom_int, health)
            },
            &Self::MovingPlatform { vertical, dist, speed } => {
                (if vertical { 82 } else { 10 }, (dist / speed) as i32, speed)
            },
            &Self::DisapearingPlatform { time_on, time_off } => {
                (83, (time_on * FRAME_RATE as f32) as i32, time_off * FRAME_RATE as f32)
            },
            &Self::SpringBoard { jump_velocity } => {
                (86, 0, jump_velocity)
            },
            &Self::BreakablePlatform { time_till_fall } => {
                (92, (time_till_fall * FRAME_RATE as f32) as i32, 0.0)
            },
            &Self::CustomizeableMoveablePlatform { bitmap_x32, target_relative, speed, start_paused, one_way } => {
                let frames_in_dir = (target_relative.0.powi(2) + target_relative.1.powi(2)).sqrt() / speed;
                let (vel_x, vel_y) = (target_relative.0 / frames_in_dir, target_relative.1 / frames_in_dir);

                let (ix, iy, fx, fy) = (
                    (vel_x.round() as i32 + 32).min(63) as u32,
                    (vel_y.round() as i32 + 32).min(63) as u32,
                    (vel_x.fract().abs() * 4.0) as u32,
                    (vel_y.fract().abs() * 4.0) as u32,
                );
                let bits = (bitmap_x32.0 & 0xf) | ((bitmap_x32.1 & 0xfff) << 4) | ix << 18 | fx << 16 | iy << 26 | fy << 24;

                (
                    107,
                    i32::from_le_bytes(bits.to_le_bytes()),
                    frames_in_dir * if start_paused { -1.0 } else { 1.0 } + if one_way { 0.5 } else { 0.0 },
                )
            },
            &Self::LockedBlock { color, consume_key } => {
                let wobj_type_id = match color {
                    KeyColor::Red => 94,
                    KeyColor::Green => 95,
                    KeyColor::Blue => 96,
                };
                (wobj_type_id, consume_key as i32, 0.0)
            },
            &Self::Vortex { attract_enemies } => {
                (105, attract_enemies as i32, 0.0)
            },
            &Self::WandRune { rune_type } => {
                let wobj_type_id = match rune_type {
                    RuneType::Ice => 19,
                    RuneType::Air => 20,
                    RuneType::Fire => 21,
                    RuneType::Lightning => 22,
                };
                (wobj_type_id, 0, 0.0)
            },
            &Self::UpgradeTrigger { trigger_type } => {
                let (wobj_type_id, custom_int) = match trigger_type {
                    UpgradeTriggerType::MaxPowerRune { skin_power_override } => {
                        if let Some(skin_id) = skin_power_override {
                            (114, skin_id as i32 - 1)
                        } else {
                            (114, 0)
                        }
                    },
                    UpgradeTriggerType::DeephausBoots => (32, 0),
                    UpgradeTriggerType::Wings => (33, 0),
                    UpgradeTriggerType::CrystalWings => (34, 0),
                    UpgradeTriggerType::Vortex => (106, 0),
                    UpgradeTriggerType::None => (140, 0),
                };
                (wobj_type_id, custom_int, 0.0)
            },
            &Self::Checkpoint => {
                checkpoints.push(spawner.pos.0);
                checkpoints.push(spawner.pos.1);
                (73, (checkpoints.len() as i32 / 2 - 1) + version.num_spawns as i32 * 2, 0.0)
            },
            &Self::Slime { flying } => (if flying { 35 } else { 2 }, 0, 0.0),
            &Self::Heavy { speed, face_left } => {
                (36, 0, speed * if face_left { -1.0 } else { 1.0 })
            },
            &Self::Dragon { space_skin } => (38, space_skin as i32, 0.0),
            &Self::BozoPin { flying_speed } => (40, 0, flying_speed),
            &Self::Bozo { mark_ii } => (if mark_ii { 72 } else { 41 }, 0, 0.0),
            &Self::SilverSlime => (42, 0, 0.0),
            &Self::LavaMonster { face_left } => (43, face_left as i32, 0.0),
            &Self::TtMinion { small } => (if small { 44 } else { 45 }, 0, 0.0),
            &Self::SlimeWalker => (46, 0, 0.0),
            &Self::MegaFish { water_level, swimming_speed } => (47, water_level, swimming_speed),
            &Self::LavaDragonHead { len, health } => (48, len as i32, health),
            &Self::TtBoss { speed } => (54, 0, speed),
            &Self::EaterBug { pop_up_speed } => (55, 0, pop_up_speed),
            &Self::SpiderWalker { speed } => (57, 0, speed),
            &Self::SpikeTrap => (59, 0, 0.0),
            &Self::SuperDragon { waypoint_id } => (63, waypoint_id as i32, 0.0),
            &Self::BozoLaserMinion { speed } => (69, 0, speed),
            &Self::SpikeGuy => (74, 0, 0.0),
            &Self::BanditGuy { speed } => (76, 0, speed),
            &Self::KamakaziSlime => (84, 0, 0.0),
            &Self::RockGuy { rock_guy_type } => {
                let (wobj_type_id, custom_int) = match rock_guy_type {
                    RockGuyType::Medium => (97, 0),
                    RockGuyType::Small1 => (98, 0),
                    RockGuyType::Small2 { face_left } => (99, face_left as i32),
                };
                (wobj_type_id, custom_int, 0.0)
            },
            &Self::RockGuySlider => (100, 0, 0.0),
            &Self::RockGuySmasher => (101, 0, 0.0),
            &Self::Wolf => (122, 0, 0.0),
            &Self::Lua { lua_wobj_type } => {
                (lua_wobj_type as i32 + 124, 0, 0.0)
            },
            &Self::Supervirus => (123, 0, 0.0),
        }
    }
}
