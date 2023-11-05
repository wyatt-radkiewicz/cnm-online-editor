use std::fmt::Display;

use crate::lparse::{Error, LParse};

use super::{super::consts::*, super::Point, super::VersionSpecs, item_type::ItemType};

/// Size of a tunes trigger object
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum TunesTriggerSize {
    #[default]
    ///
    Small,
    ///
    Big,
    ///
    VeryBig,
}

/// Rune type
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum RuneType {
    ///
    #[default]
    Fire,
    ///
    Ice,
    ///
    Air,
    ///
    Lightning,
}

/// Talking Teady Boss Node Type
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum TtNodeType {
    ///
    #[default]
    NormalTrigger,
    ///
    ChaseTrigger,
    /// Maximum of 128 waypoints in version 1 of level spec
    Waypoint(i32),
    /// This version is for bozo
    BozoWaypoint,
}

/// Push/Conveyor type
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum PushZoneType {
    ///
    #[default]
    Horizontal,
    ///
    Vertical,
    ///
    HorizontalSmall,
}

/// Color of a locked key block
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum KeyColor {
    ///
    #[default]
    Red,
    ///
    Green,
    ///
    Blue,
}

///
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum RockGuyType {
    ///
    #[default]
    Medium,
    ///
    Small1,
    ///
    Small2 {
        ///
        face_left: bool,
    },
}

///
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum UpgradeTriggerType {
    ///
    Wings,
    ///
    DeephausBoots,
    ///
    #[default]
    CrystalWings,
    ///
    Vortex,
    ///
    MaxPowerRune {
        /// Will override the player skin and give them a specific upgrade power type
        skin_power_override: Option<u8>,
    },
    ///
    None,
}

/// Used in certain World Object (Wobj) types
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Default)]
pub struct Teleport {
    /// Name of the teleport, look in [`crate::lparse::level_data::VersionSpecs`] to see how long it can be
    pub name: String,
    /// Negative numbers don't do anything special, its just they can be there in CNM Online
    pub cost: i32,
    /// Where it will teleport you
    pub loc: Point,
}

impl Display for Teleport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Teleport")
    }
}

impl Teleport {
    fn from_lparse(cnms: &LParse, version: &VersionSpecs, index: usize) -> Result<Self, Error> {
        let start = index * version.teleport_name_size;
        let end = start + version.teleport_name_size;
        let name = match String::from_utf8(
            cnms.try_get_entry("TI_NAME")?.try_get_u8()?[start..end]
                .iter()
                .cloned()
                .collect(),
        ) {
            Ok(s) => s.trim_end_matches('\0').to_string(),
            Err(_) => {
                return Err(Error::Corrupted(format!(
                    "Corrupted teleport entry name. Teleport ID: {index}"
                )))
            }
        };

        let loc = &cnms.try_get_entry("TI_POS")?.try_get_f32()?[index * 2..index * 2 + 2];

        Ok(Teleport {
            name,
            cost: cnms.try_get_entry("TI_COST")?.try_get_i32()?[index],
            loc: Point(loc[0], loc[1]),
        })
    }
}

///
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum BackgroundSwitcherShape {
    ///
    #[default]
    Small,
    ///
    Horizontal,
    ///
    Vertical,
}

impl Default for WobjType {
    fn default() -> Self {
        Self::Slime { flying: false }
    }
}

///
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum CustomizableMovingPlatformType {
    ///
    #[default]
    Normal,
    ///
    OneWay,
    ///
    Despawn
}

impl CustomizableMovingPlatformType {
    ///
    pub fn to_float_id(&self) -> f32 {
        match self {
            &Self::Normal => 0.0,
            &Self::OneWay => 0.5,
            &Self::Despawn => 0.25,
        }
    }

    ///
    pub fn from_float_id(id: f32) -> Self {
        if id < 0.1 { Self::Normal }
        else if id < 0.3 { Self::Despawn }
        else if id < 0.6 { Self::OneWay }
        else { Self::Normal }
    }
}

/// Type of a CNM Online object (and what will spawn from a spawner)
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub enum WobjType {
    ///
    Teleport(Teleport),
    ///
    Slime {
        ///
        flying: bool,
    },
    ///
    TunesTrigger {
        ///
        size: TunesTriggerSize,
        ///
        music_id: u32,
    },
    ///
    PlayerSpawn,
    ///
    TextSpawner {
        ///
        dialoge_box: bool,
        ///
        despawn: bool,
        ///
        text: String,
    },
    ///
    MovingPlatform {
        ///
        vertical: bool,
        ///
        dist: f32,
        ///
        speed: f32,
    },
    ///
    BreakableWall {
        ///
        skin_id: Option<u8>,
        ///
        health: f32,
    },
    ///
    BackgroundSwitcher {
        ///
        shape: BackgroundSwitcherShape,
        ///
        enabled_layers: std::ops::Range<u32>,
    },
    ///
    DroppedItem {
        ///
        item: ItemType,
    },
    ///
    WandRune {
        ///
        rune_type: RuneType,
    },
    ///
    Heavy {
        ///
        speed: f32,
        ///
        face_left: bool,
    },
    ///
    Dragon {
        ///
        space_skin: bool,
    },
    ///
    BozoPin {
        ///
        flying_speed: f32,
    },
    ///
    Bozo {
        ///
        mark_ii: bool,
    },
    ///
    SilverSlime,
    ///
    LavaMonster {
        ///
        face_left: bool,
    },
    ///
    TtMinion {
        ///
        small: bool,
    },
    ///
    SlimeWalker,
    ///
    MegaFish {
        ///
        water_level: i32,
        ///
        swimming_speed: f32,
    },
    ///
    LavaDragonHead {
        /// Can only be 32 long
        len: u32,
        ///
        health: f32,
    },
    ///
    TtNode {
        ///
        node_type: TtNodeType,
    },
    ///
    TtBoss {
        ///
        speed: f32,
    },
    ///
    EaterBug {
        ///
        pop_up_speed: f32,
    },
    ///
    SpiderWalker {
        ///
        speed: f32,
    },
    ///
    SpikeTrap,
    ///
    RotatingFireColunmPiece {
        /// Pieces should be put together horizontally in the original editor,
        /// And this only specifies the x axis of the origin point of the rotating fire column
        origin_x: i32,
        ///
        degrees_per_second: f32,
    },
    ///
    MovingFire {
        ///
        vertical: bool,
        ///
        dist: i32,
        ///
        speed: f32,
        ///
        despawn: bool,
    },
    ///
    SuperDragon {
        /// Can only have up to 16 waypoint ids
        waypoint_id: u8,
    },
    ///
    SuperDragonLandingZone {
        /// Can only have up to 16 waypoints
        waypoint_id: u8,
    },
    ///
    BozoLaserMinion {
        ///
        speed: f32,
    },
    ///
    Checkpoint {
        ///
        checkpoint_num: u8,
    },
    ///
    SpikeGuy,
    ///
    BanditGuy {
        ///
        speed: f32,
    },
    ///
    PushZone {
        ///
        push_zone_type: PushZoneType,
        ///
        push_speed: f32,
    },
    ///
    VerticalWindZone {
        ///
        acceleration: f32,
    },
    ///
    DisapearingPlatform {
        ///
        time_on: f32,
        ///
        time_off: f32,
        ///
        starts_on: bool,
    },
    ///
    KamakaziSlime,
    ///
    SpringBoard {
        ///
        jump_velocity: f32,
    },
    ///
    Jumpthrough {
        ///
        big: bool,
    },
    ///
    BreakablePlatform {
        ///
        time_till_fall: f32,
    },
    ///
    LockedBlock {
        ///
        color: KeyColor,
        ///
        consume_key: bool,
    },
    ///
    RockGuy {
        ///
        rock_guy_type: RockGuyType,
    },
    ///
    RockGuySlider,
    ///
    RockGuySmasher,
    ///
    HealthSetTrigger {
        ///
        target_health: f32,
    },
    ///
    Vortex {
        ///
        attract_enemies: bool,
    },
    ///
    CustomizeableMoveablePlatform {
        ///
        bitmap_x32: (u32, u32),
        ///
        target_relative: Point,
        ///
        speed: f32,
        ///
        start_paused: bool,
        ///
        ty: CustomizableMovingPlatformType,
    },
    ///
    GraphicsChangeTrigger {
        ///
        gfx_file: String,
    },
    ///
    BossBarInfo {
        ///
        boss_name: String,
    },
    ///
    BgSpeed {
        ///
        vertical_axis: bool,
        ///
        layer: u32,
        ///
        speed: f32,
    },
    ///
    BgTransparency {
        ///
        layer: u32,
        ///
        transparency: u8,
    },
    ///
    TeleportTrigger1 {
        ///
        link_id: u32,
        ///
        delay_secs: f32,
    },
    ///
    TeleportArea1 {
        ///
        link_id: u32,
        ///
        loc: Point,
    },
    ///
    SfxPoint {
        ///
        sound_id: u32,
    },
    ///
    Wolf,
    ///
    Supervirus,
    ///
    Lua {
        ///
        lua_wobj_type: u8,
    },
    ///
    UpgradeTrigger {
        ///
        trigger_type: UpgradeTriggerType,
    },
    ///
    FinishTrigger {
        ///
        next_level: String,
        ///
        extra_unlocked_level: Option<String>,
        ///
        is_secret: bool,
    },
    ///
    GravityTrigger {
        ///
        gravity: f32,
    },
    ///
    SkinUnlock {
        ///
        id: u8,
    },
    ///
    CoolPlatform {
        ///
        time_off_before: u8,
        ///
        time_on: u8,
        ///
        time_off_after: u8,
    },
    ///
    TeleportArea2 {
        ///
        loc: Point,
        ///
        start_activated: bool,
        ///
        teleport_players: bool,
        ///
        link_id: u32,
    },
}

impl WobjType {
    pub(super) fn from_lparse(
        cnms: &LParse,
        version: &VersionSpecs,
        index: usize,
        ignore_warnings: bool,
    ) -> Result<Self, Error> {
        let wobj_type_id = cnms.try_get_entry("SP_TYPE")?.try_get_i32()?[index];
        let custom_int = cnms.try_get_entry("SP_CI")?.try_get_i32()?[index];
        let custom_float = cnms.try_get_entry("SP_CF")?.try_get_f32()?[index];

        match wobj_type_id {
            1 => Ok(Self::Teleport(Teleport::from_lparse(
                cnms,
                version,
                custom_int as usize,
            )?)),
            120 => {
                let loc = Teleport::from_lparse(cnms, version, custom_int as usize)?.loc;
                Ok(Self::TeleportArea1 {
                    link_id: custom_float as u32,
                    loc,
                })
            }
            6 => Ok(Self::TunesTrigger {
                size: TunesTriggerSize::Small,
                music_id: custom_int as u32,
            }),
            7 => Ok(Self::TunesTrigger {
                size: TunesTriggerSize::Big,
                music_id: custom_int as u32,
            }),
            89 => Ok(Self::TunesTrigger {
                size: TunesTriggerSize::VeryBig,
                music_id: custom_int as u32,
            }),
            8 => Ok(Self::PlayerSpawn),
            9 | 108 => {
                // Ending Text/Dialoge Box
                let mut text = "".to_string();
                for i in (custom_int & 0x00ff_ffff) as usize..=custom_float as usize {
                    text += (super::get_ending_text_line(cnms, version, i).unwrap_or_default()
                        + "\n")
                        .as_str();
                }

                Ok(Self::TextSpawner {
                    dialoge_box: wobj_type_id == 108,
                    despawn: (custom_int & (0xff00_0000u32 as i32)) != 0,
                    text,
                })
            }
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
                    None if !ignore_warnings => {
                        return Err(Error::Corrupted(format!("Corrupt item ID {custom_int}!")))
                    }
                    None => ItemType::Apple,
                },
            }),
            14 => Ok(Self::DroppedItem {
                item: ItemType::Knife,
            }),
            15 => Ok(Self::DroppedItem {
                item: ItemType::Apple,
            }),
            4 => Ok(Self::DroppedItem {
                item: ItemType::Shotgun,
            }),
            51 | 52 | 53 | 146 => Ok(Self::TtNode {
                node_type: match wobj_type_id {
                    51 => TtNodeType::ChaseTrigger,
                    52 => TtNodeType::NormalTrigger,
                    53 => TtNodeType::Waypoint(custom_int),
                    146 => TtNodeType::BozoWaypoint,
                    _ => panic!("Unknown TT Node type!"),
                },
            }),
            60 => Ok(Self::RotatingFireColunmPiece {
                origin_x: custom_int,
                degrees_per_second: custom_float * FRAME_RATE as f32,
            }),
            61 | 62 => Ok(Self::MovingFire {
                vertical: wobj_type_id == 61,
                dist: custom_int,
                speed: custom_float.abs(),
                despawn: custom_float < 0.0,
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
            79 => Ok(Self::VerticalWindZone {
                acceleration: custom_float,
            }),
            64 => Ok(Self::SuperDragonLandingZone {
                waypoint_id: custom_int as u8,
            }),
            90 | 91 => Ok(Self::Jumpthrough {
                big: wobj_type_id == 91,
            }),
            104 => Ok(Self::HealthSetTrigger {
                target_health: custom_float,
            }),
            109 => Ok(Self::GraphicsChangeTrigger {
                gfx_file: super::get_ending_text_line(cnms, version, custom_int as usize)?,
            }),
            115 => Ok(Self::BossBarInfo {
                boss_name: super::get_ending_text_line(cnms, version, custom_int as usize)?,
            }),
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
            121 => Ok(Self::SfxPoint {
                sound_id: custom_int as u32,
            }),
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
                dist: (custom_float * custom_int as f32).abs(),
                speed: custom_float,
            }),
            83 => Ok(Self::DisapearingPlatform {
                time_on: ((custom_int / FRAME_RATE) as f32).abs(),
                time_off: custom_float / FRAME_RATE as f32,
                starts_on: custom_int >= 0,
            }),
            86 => Ok(Self::SpringBoard {
                jump_velocity: custom_float,
            }),
            92 => Ok(Self::BreakablePlatform {
                time_till_fall: custom_int as f32 / FRAME_RATE as f32,
            }),
            107 => {
                // Customizable moving platform
                let masked = custom_int as u32 >> 16 & 0xff;
                let fpart_x = (1.0 / 16.0) * (masked & 0xf) as f32;
                let mut vel_x = ((masked >> 4 & 0xf) as i32 - 8) as f32;
                vel_x += if vel_x < 0.0 { -fpart_x } else { fpart_x };
                let masked = custom_int as u32 >> 24 & 0xff;
                let fpart_y = (1.0 / 16.0) * (masked & 0xf) as f32;
                let mut vel_y = ((masked >> 4 & 0xf) as i32 - 8) as f32;
                vel_y += if vel_y < 0.0 { -fpart_y } else { fpart_y };
                let bitmap_x = custom_int & 0xf;
                let bitmap_y = custom_int >> 4 & 0xfff;
                let time = custom_float.floor().abs();

                Ok(Self::CustomizeableMoveablePlatform {
                    bitmap_x32: (bitmap_x as u32, bitmap_y as u32),
                    target_relative: Point(vel_x * time, vel_y * time),
                    speed: (vel_x.powi(2) + vel_y.powi(2)).sqrt(),//if vel_x.abs().max(vel_y.abs()) == vel_x.abs() { vel_x.abs() } else { vel_y.abs() },
                    start_paused: custom_float < 0.0,
                    ty: CustomizableMovingPlatformType::from_float_id(custom_float.abs().fract())
                    //one_way: custom_float.fract().abs() > 0.1,
                })
            }
            94 | 95 | 96 => Ok(Self::LockedBlock {
                color: match wobj_type_id {
                    94 => KeyColor::Red,
                    95 => KeyColor::Green,
                    96 => KeyColor::Blue,
                    _ => panic!("Unknown locked block key color!"),
                },
                consume_key: custom_int != 0,
            }),
            105 => Ok(Self::Vortex {
                attract_enemies: custom_int != 0,
            }),
            19 | 20 | 21 | 22 | 23 => Ok(Self::WandRune {
                rune_type: match wobj_type_id {
                    19 => RuneType::Ice,
                    20 => RuneType::Air,
                    21 | 23 => RuneType::Fire,
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
                        },
                    },
                    32 => UpgradeTriggerType::DeephausBoots,
                    33 => UpgradeTriggerType::Wings,
                    34 => UpgradeTriggerType::CrystalWings,
                    106 => UpgradeTriggerType::Vortex,
                    140 => UpgradeTriggerType::None,
                    _ => panic!("Unknown upgrade trigger type!"),
                },
            }),
            73 => Ok(Self::Checkpoint {
                checkpoint_num: (custom_int - version.get_num_spawns() as i32 * 2).clamp(0, 255)
                    as u8,
            }),
            2 | 35 => Ok(Self::Slime {
                flying: wobj_type_id == 35,
            }),
            36 => Ok(Self::Heavy {
                speed: custom_float.abs(),
                face_left: custom_float < 0.0,
            }),
            38 => Ok(Self::Dragon {
                space_skin: custom_int != 0,
            }),
            40 => Ok(Self::BozoPin {
                flying_speed: custom_float,
            }),
            41 | 72 => Ok(Self::Bozo {
                mark_ii: wobj_type_id == 72,
            }),
            42 => Ok(Self::SilverSlime),
            43 => Ok(Self::LavaMonster {
                face_left: custom_int != 0,
            }),
            44 | 45 => Ok(Self::TtMinion {
                small: custom_int == 44,
            }),
            46 => Ok(Self::SlimeWalker),
            47 => Ok(Self::MegaFish {
                water_level: custom_int,
                swimming_speed: custom_float,
            }),
            48 => Ok(Self::LavaDragonHead {
                len: if custom_int > 32 && !ignore_warnings {
                    return Err(Error::Corrupted(
                        "Lava dragon boss has too much segments!".to_string(),
                    ));
                } else {
                    custom_int as u32
                },
                health: custom_float,
            }),
            54 => Ok(Self::TtBoss {
                speed: custom_float,
            }),
            55 => Ok(Self::EaterBug {
                pop_up_speed: custom_float,
            }),
            57 => Ok(Self::SpiderWalker {
                speed: custom_float,
            }),
            59 => Ok(Self::SpikeTrap),
            63 => Ok(Self::SuperDragon {
                waypoint_id: custom_int as u8,
            }),
            69 => Ok(Self::BozoLaserMinion {
                speed: custom_float,
            }),
            74 => Ok(Self::SpikeGuy),
            76 => Ok(Self::BanditGuy {
                speed: custom_float,
            }),
            84 => Ok(Self::KamakaziSlime),
            97 | 98 | 99 => Ok(Self::RockGuy {
                rock_guy_type: match wobj_type_id {
                    97 => RockGuyType::Medium,
                    98 => RockGuyType::Small1,
                    99 => RockGuyType::Small2 {
                        face_left: custom_int != 0,
                    },
                    _ => panic!("Unknown rock guy type!"),
                },
            }),
            100 => Ok(Self::RockGuySlider),
            101 => Ok(Self::RockGuySmasher),
            122 => Ok(Self::Wolf),
            123 => Ok(Self::Supervirus),
            141 => Ok(Self::FinishTrigger {
                next_level: super::get_ending_text_line(cnms, version, (custom_int & 0xff) as usize).unwrap_or("".to_string()),
                extra_unlocked_level: if custom_float as i32 > 0 {
                    Some(super::get_ending_text_line(cnms, version, custom_float as usize - 1).unwrap_or("".to_string()))
                } else {
                    None
                },
                is_secret: (custom_int & 0xf00) != 0,
            }),
            143 => Ok(Self::GravityTrigger {
                gravity: custom_float,
            }),
            148 => Ok(Self::SkinUnlock {
                id: custom_int as u8,
            }),
            149 => Ok(Self::CoolPlatform {
                time_off_before: (custom_int & 0xff) as u8,
                time_on: (custom_int >> 8 & 0xff) as u8,
                time_off_after: (custom_int >> 16 & 0xff) as u8,
            }),
            150 => {
                let loc = Teleport::from_lparse(cnms, version, custom_int as usize)?.loc;
                Ok(Self::TeleportArea2 {
                    loc,
                    start_activated: custom_int & 0x10000 != 0,
                    teleport_players: custom_int & 0x20000 == 0,
                    link_id: custom_float as u32,
                })
            }
            _ if wobj_type_id >= 124 && wobj_type_id <= 139 => Ok(Self::Lua {
                lua_wobj_type: (wobj_type_id - 124) as u8,
            }),
            _ if !ignore_warnings => Err(Error::Corrupted(format!(
                "Unknown wobj type {wobj_type_id}"
            ))),
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
            }
            &Self::TeleportArea1 { link_id, loc } => {
                teleports.push(Teleport {
                    name: "_TELEAREA".to_string(),
                    cost: 0,
                    loc,
                });
                (120, teleports.len() as i32 - 1, link_id as f32)
            }
            &Self::TunesTrigger { size, music_id } => {
                let wobj_type_id = match size {
                    TunesTriggerSize::Small => 6,
                    TunesTriggerSize::Big => 7,
                    TunesTriggerSize::VeryBig => 89,
                };
                (wobj_type_id, music_id as i32, 0.0)
            }
            &Self::PlayerSpawn => {
                spawns.push(spawner.pos.0);
                spawns.push(spawner.pos.1);
                (8, spawns.len() as i32 / 2 - 1, 0.0)
            }
            &Self::TextSpawner {
                dialoge_box,
                despawn,
                ref text,
            } => {
                let wobj_type_id = if dialoge_box { 108 } else { 9 };
                let (start, num_lines) = if let Some(pos) = ending_text
                    .iter()
                    .enumerate()
                    .position(|(mut idx, _)| {
                        for line in text.lines() {
                            if ending_text.get(idx).unwrap_or(&String::new()) != line {
                                return false;
                            }
                            idx += 1;
                        }
                        return true;
                    }) {
                    (pos, text.lines().count())
                } else {
                    let start = ending_text.len();
                    let mut len = 0;
                    for line in text.lines() {
                        if ending_text.len() == version.title_ending_text_line {
                            ending_text.push("".to_string()); // Can't use the line that is used for the title!
                            len += 1;
                        }
                        ending_text.push(line.trim_end().to_string());
                        len += 1;
                    }
                    (start, len)
                };
                //println!("{start} <- start");

                (wobj_type_id, start as i32 | if despawn { 0x0100_0000u32 as i32 } else { 0 }, (start + num_lines - 1) as f32)
            }
            &Self::BackgroundSwitcher {
                shape,
                ref enabled_layers,
            } => {
                let wobj_type_id = match shape {
                    BackgroundSwitcherShape::Small => 12,
                    BackgroundSwitcherShape::Horizontal => 87,
                    BackgroundSwitcherShape::Vertical => 88,
                };
                (
                    wobj_type_id,
                    enabled_layers.start as i32,
                    enabled_layers.end as f32,
                )
            }
            &Self::DroppedItem { item } => (16, item.get_item_id() as i32, 0.0),
            &Self::TtNode { node_type } => {
                let (wobj_type_id, custom_int) = match node_type {
                    TtNodeType::ChaseTrigger => (51, 0),
                    TtNodeType::NormalTrigger => (52, 0),
                    TtNodeType::Waypoint(waypoint_id) => (53, waypoint_id),
                    TtNodeType::BozoWaypoint => (146, 0),
                };
                (wobj_type_id, custom_int, 0.0)
            }
            &Self::RotatingFireColunmPiece {
                origin_x,
                degrees_per_second,
            } => (60, origin_x, degrees_per_second / FRAME_RATE as f32),
            &Self::MovingFire {
                vertical,
                dist,
                speed,
                despawn,
            } => {
                let wobj_type_id = if vertical { 61 } else { 62 };
                (wobj_type_id, dist, speed * if despawn { -1.0 } else { 1.0 })
            }
            &Self::PushZone {
                push_zone_type,
                push_speed,
            } => {
                let wobj_type_id = match push_zone_type {
                    PushZoneType::Horizontal => 77,
                    PushZoneType::Vertical => 78,
                    PushZoneType::HorizontalSmall => 80,
                };
                (wobj_type_id, 0, push_speed)
            }
            &Self::VerticalWindZone { acceleration } => (79, 0, acceleration),
            &Self::SuperDragonLandingZone { waypoint_id } => (64, waypoint_id as i32, 0.0),
            &Self::Jumpthrough { big } => (if big { 91 } else { 90 }, 0, 0.0),
            &Self::HealthSetTrigger { target_health } => (104, 0, target_health),
            &Self::GraphicsChangeTrigger { ref gfx_file } => {
                let start = if let Some(pos) = ending_text.iter().position(|line| line == gfx_file) {
                    pos
                } else {
                    if ending_text.len() == version.title_ending_text_line {
                        ending_text.push("".to_string());
                    }
                    ending_text.push(gfx_file.clone());
                    ending_text.len() - 1
                };
                (109, start as i32, 0.0)
            }
            &Self::BossBarInfo { ref boss_name } => {
                let start = if let Some(pos) = ending_text.iter().position(|line| line == boss_name) {
                    pos
                } else {
                    if ending_text.len() == version.title_ending_text_line {
                        ending_text.push("".to_string());
                    }
                    ending_text.push(boss_name.clone());
                    ending_text.len() - 1
                };
                (115, start as i32, 0.0)
            }
            &Self::BgSpeed {
                vertical_axis,
                layer,
                speed,
            } => (if vertical_axis { 117 } else { 116 }, layer as i32, speed),
            &Self::BgTransparency {
                layer,
                transparency,
            } => (118, layer as i32, transparency as f32),
            &Self::TeleportTrigger1 {
                link_id,
                delay_secs,
            } => (119, link_id as i32, delay_secs),
            &Self::SfxPoint { sound_id } => (121, sound_id as i32, 0.0),
            &Self::BreakableWall { skin_id, health } => {
                let (wobj_type_id, custom_int) = match skin_id {
                    Some(skin_id) => (93, skin_id as i32),
                    None => (11, 0),
                };
                (wobj_type_id, custom_int, health)
            }
            &Self::MovingPlatform {
                vertical,
                dist,
                speed,
            } => (
                if vertical { 82 } else { 10 },
                (dist / speed).abs() as i32,
                speed,
            ),
            &Self::DisapearingPlatform {
                time_on,
                time_off,
                starts_on,
            } => (
                83,
                (time_on * FRAME_RATE as f32) as i32 * if starts_on { -1 } else { 1 },
                time_off * FRAME_RATE as f32,
            ),
            &Self::SpringBoard { jump_velocity } => (86, 0, jump_velocity),
            &Self::BreakablePlatform { time_till_fall } => {
                (92, (time_till_fall * FRAME_RATE as f32) as i32, 0.0)
            }
            &Self::CustomizeableMoveablePlatform {
                bitmap_x32,
                target_relative,
                speed,
                start_paused,
                ty,
            } => {
                let mut frames_in_dir = ((target_relative.0.powi(2) + target_relative.1.powi(2)).sqrt() / speed).ceil();//target_relative.0.abs().max(target_relative.1.abs()) / speed;
                if frames_in_dir == 0.0 {
                    frames_in_dir = 1.0;
                }

                let (mut vel_x, mut vel_y) = (
                    target_relative.0 / frames_in_dir,
                    target_relative.1 / frames_in_dir,
                );
                if vel_x.fract().abs() > 31.0 / 32.0 { vel_x = vel_x.round(); }
                if vel_y.fract().abs() > 31.0 / 32.0 { vel_y = vel_y.round(); }

                let (ix, iy, fx, fy) = (
                    ((vel_x.round() as i32 + 8) as u32).min(15),
                    ((vel_y.round() as i32 + 8) as u32).min(15),
                    (vel_x.fract().abs() * 16.0) as u32,
                    (vel_y.fract().abs() * 16.0) as u32,
                );
                
                let bits = (bitmap_x32.0 & 0xf)
                    | ((bitmap_x32.1 & 0xfff) << 4)
                    | ix << (16+4)
                    | fx << 16
                    | iy << (24+4)
                    | fy << 24;
                //println!("{ix} {iy} {fx} {fy} {bits} {}", i32::from_le_bytes(bits.to_le_bytes()));
                //println!("dist: {}", ((target_relative.0.powi(2) + target_relative.1.powi(2)).sqrt()));
                //println!("vel_x: {}, vel_x * frames_in_dir: {}", vel_x, vel_x * frames_in_dir);

                (
                    107,
                    i32::from_le_bytes(bits.to_le_bytes()),
                    frames_in_dir.round() * if start_paused { -1.0 } else { 1.0 }
                        + ty.to_float_id(),
                )
            }
            &Self::LockedBlock { color, consume_key } => {
                let wobj_type_id = match color {
                    KeyColor::Red => 94,
                    KeyColor::Green => 95,
                    KeyColor::Blue => 96,
                };
                (wobj_type_id, consume_key as i32, 0.0)
            }
            &Self::Vortex { attract_enemies } => (105, attract_enemies as i32, 0.0),
            &Self::WandRune { rune_type } => {
                let wobj_type_id = match rune_type {
                    RuneType::Ice => 19,
                    RuneType::Air => 20,
                    RuneType::Fire => 21,
                    RuneType::Lightning => 22,
                };
                (wobj_type_id, 0, 0.0)
            }
            &Self::UpgradeTrigger { trigger_type } => {
                let (wobj_type_id, custom_int) = match trigger_type {
                    UpgradeTriggerType::MaxPowerRune {
                        skin_power_override,
                    } => {
                        if let Some(skin_id) = skin_power_override {
                            (114, skin_id as i32 - 1)
                        } else {
                            (114, 0)
                        }
                    }
                    UpgradeTriggerType::DeephausBoots => (32, 0),
                    UpgradeTriggerType::Wings => (33, 0),
                    UpgradeTriggerType::CrystalWings => (34, 0),
                    UpgradeTriggerType::Vortex => (106, 0),
                    UpgradeTriggerType::None => (140, 0),
                };
                (wobj_type_id, custom_int, 0.0)
            }
            &Self::Checkpoint { checkpoint_num } => {
                if checkpoints.len() < checkpoint_num as usize * 2 + 2 {
                    checkpoints.resize(checkpoint_num as usize * 2 + 2, f32::NAN);
                }
                checkpoints[checkpoint_num as usize * 2 + 0] = spawner.pos.0;
                checkpoints[checkpoint_num as usize * 2 + 1] = spawner.pos.1;
                (
                    73,
                    checkpoint_num as i32 + version.num_spawns as i32 * 2,
                    0.0,
                )
            }
            &Self::Slime { flying } => (if flying { 35 } else { 2 }, 0, 0.0),
            &Self::Heavy { speed, face_left } => {
                (36, 0, speed * if face_left { -1.0 } else { 1.0 })
            }
            &Self::Dragon { space_skin } => (38, space_skin as i32, 0.0),
            &Self::BozoPin { flying_speed } => (40, 0, flying_speed),
            &Self::Bozo { mark_ii } => (if mark_ii { 72 } else { 41 }, 0, 0.0),
            &Self::SilverSlime => (42, 0, 0.0),
            &Self::LavaMonster { face_left } => (43, face_left as i32, 0.0),
            &Self::TtMinion { small } => (if small { 44 } else { 45 }, 0, 0.0),
            &Self::SlimeWalker => (46, 0, 0.0),
            &Self::MegaFish {
                water_level,
                swimming_speed,
            } => (47, water_level, swimming_speed),
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
            }
            &Self::RockGuySlider => (100, 0, 0.0),
            &Self::RockGuySmasher => (101, 0, 0.0),
            &Self::Wolf => (122, 0, 0.0),
            &Self::Lua { lua_wobj_type } => (lua_wobj_type as i32 + 124, 0, 0.0),
            &Self::Supervirus => (123, 0, 0.0),
            &Self::FinishTrigger { ref next_level, ref extra_unlocked_level, is_secret } => {
                let start = if let Some(pos) = ending_text.iter().position(|line| line == next_level) {
                    pos
                } else {
                    if ending_text.len() == version.title_ending_text_line {
                        ending_text.push("".to_string());
                    }
                    ending_text.push(next_level.clone());
                    ending_text.len() - 1
                };
                let extra_start = if let Some(extra_level) = extra_unlocked_level {
                    if let Some(pos) = ending_text.iter().position(|line| line == extra_level) {
                        (pos + 1) as f32
                    } else {
                        if ending_text.len() == version.title_ending_text_line {
                            ending_text.push("".to_string());
                        }
                        ending_text.push(extra_level.clone());
                        (ending_text.len() - 1 + 1) as f32
                    }
                } else {
                    0.0
                };
                (141, start as i32 | if is_secret { 0xf00 } else { 0 }, extra_start)
            },
            &Self::GravityTrigger { gravity } => (143, 0, gravity),
            &Self::SkinUnlock { id } => (148, id as i32, 0.0),
            &Self::CoolPlatform { time_off_before, time_on, time_off_after } => {
                (
                    149,
                    (time_off_after as i32) |
                    ((time_on as i32) << 8) |
                    ((time_off_before as i32) << 16),
                    0.0
                )
            }
            &Self::TeleportArea2 { link_id, loc, teleport_players, start_activated } => {
                teleports.push(Teleport {
                    name: "_TELEARE2".to_string(),
                    cost: 0,
                    loc,
                });
                let teleport_players_bit = if teleport_players { 0x20000 } else { 0x00000 };
                let start_activated_bit = if start_activated { 0x10000 } else { 0x00000 };
                (150, (teleports.len() as i32 - 1) | teleport_players_bit | start_activated_bit, link_id as f32)
            }
        }
    }
}
