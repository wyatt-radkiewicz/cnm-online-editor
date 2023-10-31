use crate::lparse::{
    Error,
    LParse,
    EntryData,
};

use super::{
    consts::FRAME_RATE,
    Point,
    VersionSpecs,
};

use wobj_type::WobjType;
use item_type::ItemType;

/// Items from CNM Online
pub mod item_type;
/// World Object (Wobj) types from CNM Online.
pub mod wobj_type;

/// In what context a CNM World Object (Wobj) will spawn in.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum SpawnerMode {
    /// Will spawn in all contexts (multi and single player)
    MultiAndSingleplayer,
    /// Will only spawn in singleplayer
    SingleplayerOnly,
    /// Will only spawn in multiplayer
    MultiplayerOnly,
    /// This will show in both single and multiplayer and will spawn an wobj for every
    /// player in the server/game, this means the max spawns will be max_spawns*player_count
    PlayerCountBased,
    /// This will not spawn in the level and will only be available in the spawner data.
    NoSpawn,
}

impl SpawnerMode {
    fn from_packed_dropped_item(dropped_item: u32) -> Option<Self> {
        let mode_id = (dropped_item & 0xff00_0000) >> 24;
        match mode_id {
            0 => Some(Self::MultiAndSingleplayer),
            0x10 => Some(Self::SingleplayerOnly),
            0x20 => Some(Self::MultiplayerOnly),
            0x30 => Some(Self::PlayerCountBased),
            4 => Some(Self::NoSpawn),
            _ => None,
        }
    }

    fn to_packed_dropped_item(&self) -> u32 {
        match self {
            &Self::MultiAndSingleplayer => 0,
            &Self::SingleplayerOnly => 1 << 28,
            &Self::MultiplayerOnly => 2 << 28,
            &Self::PlayerCountBased => 3 << 28,
            &Self::NoSpawn => 4 << 24,
        }
    }
}

/// Criteria for how a World Object (Wobj) spawns.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy)]
pub struct SpawningCriteria {
    /// How many seconds between spawns
    pub spawn_delay_secs: f32,
    /// In what context it will spawn in
    pub mode: SpawnerMode,
    /// How many objects it will have spawned in at any given time
    pub max_concurrent_spawns: u32,
}

/// A template for a object to spawn in cnm
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct Spawner {
    /// Where it will spawn
    pub pos: Point,
    /// What object type (World object or Wobj)
    pub type_data: WobjType,
    /// In what cirteria it will spawn in
    pub spawning_criteria: SpawningCriteria,
    /// What item it will drop once it's destroyed
    pub dropped_item: Option<ItemType>,
    /// What spawner group its a part of (spawners in these
    /// groups will be guarenteed to spawn on the exact same tick)
    pub spawner_group: Option<u8>,
}

impl Spawner {
    pub(crate) fn from_lparse(cnms: &LParse, version: &VersionSpecs, index: usize, ignore_warnings: bool) -> Result<Self, Error> {
        let position = &cnms.try_get_entry("SP_POS")?.try_get_f32()?[index * 2..index * 2 + 2];
        let duration = cnms.try_get_entry("SP_DURATION")?.try_get_i32()?[index];
        let max_spawns = cnms.try_get_entry("SP_MAX")?.try_get_i32()?[index];
        let dropped_item = cnms.try_get_entry("SP_DITEM")?.try_get_u32()?[index];
        let spawner_group = if let Ok(entry) = cnms.try_get_entry("SP_GROUP") {
            let id = entry.try_get_u8()?[index];
            if id == 0xff { None }
            else { Some(id) }
        } else {
            None
        };

        Ok(Self {
            pos: Point(position[0], position[1]),
            type_data: WobjType::from_lparse(cnms, version, index, ignore_warnings)?,
            spawning_criteria: SpawningCriteria {
                spawn_delay_secs: duration as f32 / FRAME_RATE as f32,
                mode: match SpawnerMode::from_packed_dropped_item(dropped_item) {
                    Some(mode) => mode,
                    None if !ignore_warnings => return Err(Error::Corrupted(format!("Corrupt spawing mode on entity id {index}."))),
                    None => SpawnerMode::MultiAndSingleplayer,
                },
                max_concurrent_spawns: max_spawns as u32,
            },
            dropped_item: ItemType::from_item_id(dropped_item),
            spawner_group,
        })
    }

    pub(super) fn save(
        &self,
        teleports: &mut Vec<wobj_type::Teleport>,
        spawns: &mut Vec<f32>,
        checkpoints: &mut Vec<f32>,
        ending_text: &mut Vec<String>,
        version: &VersionSpecs,
    ) -> (Point, i32, i32, i32, i32, f32, u32, u8) {
        let wobj_type_info = self.type_data.serialize(
            teleports,
            spawns,
            checkpoints,
            ending_text,
            self,
            version,
        );
        (
            self.pos,
            wobj_type_info.0,
            (self.spawning_criteria.spawn_delay_secs * FRAME_RATE as f32) as i32,
            self.spawning_criteria.max_concurrent_spawns as i32,
            wobj_type_info.1,
            wobj_type_info.2,
            if let Some(i) = self.dropped_item { i.get_item_id() } else { 0 } | self.spawning_criteria.mode.to_packed_dropped_item(),
            if let Some(group) = self.spawner_group { group } else { 0xff },
        )
    }
}

pub(crate) fn get_ending_text_line(cnms: &LParse, version: &VersionSpecs, index: usize) -> Result<String, Error> {
    let (start, end) = (index * version.ending_text_line_len, (index + 1) * version.ending_text_line_len);
    let buffer = cnms.try_get_entry("ENDINGTEXT")?.try_get_u8()?[start..end].iter().cloned().collect();
    match String::from_utf8(buffer) {
        Ok(s) => Ok(s.trim_end_matches('\0').to_string()),
        Err(_) => Err(Error::Corrupted(format!("Can't find ending text entry id {index} probably because it is out of the range 0..{}.", version.ending_text_lines))),
    }
}

fn set_ending_text_line<S: AsRef<str>>(cnms: &mut LParse, version: &VersionSpecs, index: usize, line: &S) {
    if index >= version.ending_text_lines {
        return;
    }
    
    let mut line = line.as_ref().to_string() + "\0".repeat(version.ending_text_line_len).as_str();
    line.truncate(version.ending_text_line_len);
    let bytes = match cnms.entries.get_mut("ENDINGTEXT") {
        Some(EntryData::U8(ref mut v)) => v,
        _ => {
            cnms.entries.insert("ENDINGTEXT".to_string(), EntryData::U8(Vec::new()));
            if let EntryData::U8(ref mut v) = cnms.entries.get_mut("ENDINGTEXT").unwrap() {
                v
            } else {
                panic!("ENDINGTEXT u8 vector not found!");
            }
        },
    };

    let (start, end) = (index * version.ending_text_line_len, (index + 1) * version.ending_text_line_len);
    if bytes.len() < end {
        bytes.resize(end, '\0' as u8);
    }

    bytes.as_mut_slice()[start..end].copy_from_slice(line.as_bytes());
}

pub(super) fn save_spawner_vec(cnms: &mut LParse, version: &VersionSpecs, level_title: String, spawners: &[Spawner]) {
    // LParse arrays
    let mut teleports = Vec::new();
    let mut spawns = Vec::new();
    let mut checkpoints = Vec::new();
    let mut lines = Vec::new();
    let mut sp_pos = Vec::new();
    let mut sp_type = Vec::new();
    let mut sp_duration = Vec::new();
    let mut sp_max = Vec::new();
    let mut sp_ci = Vec::new();
    let mut sp_cf = Vec::new();
    let mut sp_ditem = Vec::new();
    let mut sp_group = Vec::new();

    // Collect the spawner data
    for spawner in spawners.iter() {
        let data = spawner.save(
            &mut teleports,
            &mut spawns,
            &mut checkpoints,
            &mut lines,
            version
        );
        sp_pos.push(data.0.0);
        sp_pos.push(data.0.1);
        sp_type.push(data.1);
        sp_duration.push(data.2);
        sp_max.push(data.3);
        sp_ci.push(data.4);
        sp_cf.push(data.5);
        sp_ditem.push(data.6);
        sp_group.push(data.7);
    }

    // Finalize things like ending text and the number of teleports
    let num_alloced_teleports = teleports.len();
    teleports.resize(version.num_teleports, wobj_type::Teleport { name: String::new(), cost: 0, loc: Point(0.0, 0.0) });
    let mut ti_name = Vec::new();
    teleports.iter().for_each(|teleport| {
        let mut name = teleport.name.to_owned() + "\0".repeat(version.teleport_name_size).as_str();
        name.truncate(version.teleport_name_size);
        ti_name.append(&mut name.as_bytes().iter().cloned().collect());
    });
    let ti_cost = teleports.iter().map(|teleport| teleport.cost).collect::<Vec<i32>>();
    let mut ti_pos = Vec::new();
    teleports.iter().for_each(|teleport| { ti_pos.push(teleport.loc.0); ti_pos.push(teleport.loc.1); });
    let ti_alloced = (0..version.num_teleports).map(|i| if i < num_alloced_teleports { 0 } else { 1 }).collect::<Vec<u8>>();
    lines.resize(version.ending_text_lines, "".to_string());
    lines[version.title_ending_text_line] = level_title;
    spawns.resize(version.num_spawns * 2, f32::INFINITY);
    checkpoints.resize(version.num_spawns * 2, f32::INFINITY);

    // Write everything
    for (idx, line) in lines.iter().enumerate() {
        set_ending_text_line(cnms, version, idx, line);
    }
    let (mut spawnx, mut spawny) = (Vec::new(), Vec::new());
    let full_spawns = spawns.iter().chain(spawns.iter().chain(checkpoints.iter())).enumerate();
    for (idx, spawn) in full_spawns.clone() {
        if idx % 2 == 0 {
            spawnx.push(*spawn);
        } else {
            spawny.push(*spawn);
        }
    }
    cnms.entries.insert("PLAYERSPAWNX".to_string(), EntryData::F32(spawnx));
    cnms.entries.insert("PLAYERSPAWNY".to_string(), EntryData::F32(spawny));
    cnms.entries.insert("TI_NAME".to_string(), EntryData::U8(ti_name));
    cnms.entries.insert("TI_COST".to_string(), EntryData::I32(ti_cost));
    cnms.entries.insert("TI_POS".to_string(), EntryData::F32(ti_pos));
    cnms.entries.insert("TI_ALLOCED".to_string(), EntryData::U8(ti_alloced));
    cnms.entries.insert("SP_POS".to_string(), EntryData::F32(sp_pos));
    cnms.entries.insert("SP_TYPE".to_string(), EntryData::I32(sp_type));
    cnms.entries.insert("SP_DURATION".to_string(), EntryData::I32(sp_duration));
    cnms.entries.insert("SP_MAX".to_string(), EntryData::I32(sp_max));
    cnms.entries.insert("SP_CI".to_string(), EntryData::I32(sp_ci));
    cnms.entries.insert("SP_CF".to_string(), EntryData::F32(sp_cf));
    cnms.entries.insert("SP_DITEM".to_string(), EntryData::U32(sp_ditem));
    cnms.entries.insert("SP_GROUP".to_string(), EntryData::U8(sp_group));
    cnms.entries.insert("NUM_SPAWNERS".to_string(), EntryData::I32(vec![spawners.len() as i32]));
}
