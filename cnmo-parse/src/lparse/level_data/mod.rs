use crate::lparse::{
    Error,
    LParse
};

use self::cnmb_types::{BackgroundLayer, TileProperties};

pub mod cnmb_types;
pub mod cnms_types;
pub mod consts;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Default, Copy, Clone)]
pub struct Duration(pub i32);

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Default, Copy, Clone)]
pub struct Point(pub f32, pub f32);

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug)]
pub struct VersionSpecs {
    version: u32,
    num_teleports: usize,
    num_spawns: usize,
    num_spawn_modes: usize,
    teleport_name_size: usize,
    max_tile_frames: usize,
    ending_text_lines: usize,
    ending_text_line_len: usize,
    background_layers: usize,
    title_ending_text_line: usize,
    preview_tile_index: usize,
}

impl VersionSpecs {
    pub fn from_version(version: u32) -> Result<Self, Error> {
        match version {
            1 => Ok(Self {
                version,
                num_teleports: 512,
                num_spawns: 128,
                num_spawn_modes: 3,
                teleport_name_size: 41,
                max_tile_frames: 32,
                ending_text_lines: 48,
                ending_text_line_len: 32,
                background_layers: 32,
                title_ending_text_line: 47,
                preview_tile_index: 256,
            }),
            _ => Err(Error::UnknownVersion(version))
        }
    }

    pub fn get_version(&self) -> u32 {
        self.version
    }

    pub fn get_num_teleports(&self) -> usize {
        self.num_teleports
    }
    
    pub fn get_num_spawns(&self) -> usize {
        self.num_spawns
    }

    pub fn get_num_spawn_modes(&self) -> usize {
        self.num_spawn_modes
    }

    pub fn get_teleport_name_size(&self) -> usize {
        self.teleport_name_size
    }

    pub fn get_max_tile_frames(&self) -> usize {
        self.max_tile_frames
    }

    pub fn get_ending_text_lines(&self) -> usize {
        self.ending_text_lines
    }

    pub fn get_ending_text_line_len(&self) -> usize {
        self.ending_text_line_len
    }

    pub fn get_background_layers(&self) -> usize {
        self.background_layers
    }
    
    pub fn get_title_ending_text_line(&self) -> usize {
        self.title_ending_text_line
    }

    pub fn get_preview_tile_index(&self) -> usize {
        self.preview_tile_index
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, PartialEq, Eq, num_derive::FromPrimitive, num_derive::ToPrimitive)]
pub enum DifficultyRating {
    Tutorial,
    ReallyEasy,
    Easy,
    Normal,
    KindaHard,
    Hard,
    Ultra,
    Extreme,
    Dealth,
    UltraDeath,
}

impl DifficultyRating {
    pub fn from_difficulty_id(id: u8) -> Option<Self> {
        num_traits::FromPrimitive::from_u8(id)
    }

    pub fn get_difficulty_id(&self) -> u8 {
        num_traits::ToPrimitive::to_u8(self).unwrap_or(3)
    }

    pub fn to_string_pretty(&self) -> String {
        match self {
            &Self::Tutorial => "Tutorial".to_string(),
            &Self::ReallyEasy => "Really Easy".to_string(),
            &Self::Easy => "Easy".to_string(),
            &Self::Normal => "Normal".to_string(),
            &Self::KindaHard => "Kinda Hard".to_string(),
            &Self::Hard => "Hard".to_string(),
            &Self::Ultra => "Ultra!".to_string(),
            &Self::Extreme => "Extreme!".to_string(),
            &Self::Dealth => "Death!!!".to_string(),
            &Self::UltraDeath => "ULTRA DEATH!".to_string(),
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug)]
pub struct LevelMetaData {
    pub title: String,
    pub subtitle: Option<String>,
    pub preview_loc: (u32, u32),
    pub difficulty_rating: DifficultyRating,
}

impl LevelMetaData {
    pub fn from_lparse(cnmb: &LParse, cnms: &LParse, version: &VersionSpecs, ignore_warnings: bool) -> Result<Self, Error> {
        let title_full = cnms_types::get_ending_text_line(cnms, version, version.title_ending_text_line)?;
        let title = title_full.split('\\').next().unwrap_or("").to_string();
        let subtitle = match title_full.split('\\').nth(1) {
            Some(s) => Some(s.to_string()),
            None => None,
        };

        let tile_properties = cnmb_types::TileProperties::from_lparse(cnmb, version, version.preview_tile_index, ignore_warnings)?;
        let preview_loc = (tile_properties.frames[0].0 as u32, tile_properties.frames[0].1 as u32);
        let difficulty_rating = DifficultyRating::from_difficulty_id(
            cnmb.try_get_entry("BP_DMG")?
                .try_get_i32()?[version.preview_tile_index] as u8
        ).unwrap_or(DifficultyRating::Normal);

        Ok(Self {
            title,
            subtitle,
            preview_loc,
            difficulty_rating,
        })
    }

    fn get_full_title(&self) -> String {
        let subtitle = "\\".to_string() + self.subtitle.as_ref().unwrap_or(&"".to_string()).as_str();
        self.title.clone() + match self.subtitle {
            Some(_) => subtitle.as_str(),
            None => "",
        }
    }

    fn get_tile_property(&self) -> cnmb_types::TileProperties {
        cnmb_types::TileProperties {
            solid: false,
            transparency: consts::CLEAR,
            damage_type: cnmb_types::DamageType::Lava(self.difficulty_rating.get_difficulty_id() as i32),
            anim_speed: Duration(1),
            frames: vec![(self.preview_loc.0 as i32, self.preview_loc.1 as i32)],
            collision_data: cnmb_types::CollisionType::Box(crate::Rect { x: 0, y: 0, w: 0, h: 0 }),
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug)]
pub struct LevelData {
    pub version: VersionSpecs,
    pub spawners: Vec<cnms_types::Spawner>,
    pub cells: cnmb_types::Cells,
    pub tile_properties: Vec<cnmb_types::TileProperties>,
    pub metadata: LevelMetaData,
    pub background_layers: Vec<BackgroundLayer>,
}

impl LevelData {
    pub fn from_version(version: u32) -> Result<Self, Error> {
        let version = VersionSpecs::from_version(version)?;
        let background_layers = (0..version.background_layers).map(|_| cnmb_types::BackgroundLayer::default()).collect();

        Ok(Self {
            version,
            spawners: Vec::new(),
            tile_properties: Vec::new(),
            cells: cnmb_types::Cells::new(512, 256),
            metadata: LevelMetaData {
                title: "Untitled".to_string(),
                subtitle: None,
                preview_loc: (0, 0),
                difficulty_rating: DifficultyRating::Normal,
            },
            background_layers,
        })
    }

    pub fn from_lparse(cnmb: &LParse, cnms: &LParse, ignore_warnings: bool) -> Result<Self, Error> {
        if cnmb.version.version != cnms.version.version {
            return Err(Error::MismatchedVersions(cnmb.version.version, cnms.version.version));
        }

        let version = VersionSpecs::from_version(cnmb.version.version)?;
        let tile_properties = Self::tile_properties_from_lparse(cnmb, &version, ignore_warnings)?;
        let cells = cnmb_types::Cells::from_lparse(cnmb, tile_properties.len())?;
        let spawners = Self::spawners_from_lparse(cnms, &version, ignore_warnings)?;
        let metadata = LevelMetaData::from_lparse(cnmb, cnms, &version, ignore_warnings)?;
        let background_layers = Self::background_layers_from_lparse(cnmb, &version)?;

        Ok(Self {
            version,
            tile_properties,
            cells,
            spawners,
            metadata,
            background_layers,
        })
    }

    pub fn save(&self, cnmb: &mut LParse, cnms: &mut LParse) {
        cnms_types::save_spawner_vec(cnms, &self.version, self.metadata.get_full_title(), &self.spawners);
        cnmb_types::save_background_vec(cnmb, &self.version, &self.background_layers);
        cnmb_types::save_tile_properties_vec(cnmb, &self.version, &self.metadata.get_tile_property(), &self.tile_properties);
        self.cells.save(cnmb, self.tile_properties.len() + 1, &self.version);
    }

    fn tile_properties_from_lparse(cnmb: &LParse, version: &VersionSpecs, ignore_warnings: bool) -> Result<Vec<cnmb_types::TileProperties>, Error> {
        let mut tile_properties = Vec::new();

        for index in 0..cnmb.try_get_entry("BLOCKS_HEADER")?.try_get_i32()?[2] as usize {
            if index == version.preview_tile_index {
                continue;
            }
            let tile = cnmb_types::TileProperties::from_lparse(cnmb, version, index, ignore_warnings)?;
            match tile {
                TileProperties {
                    damage_type: cnmb_types::DamageType::None,
                    anim_speed: Duration(1),
                    frames,
                    collision_data: cnmb_types::CollisionType::Box(crate::Rect { x: 0, y: 0, w: 32, h: 32 }),
                    ..
                } if frames.len() == 1 && frames[0] == (0, 0) => {},
                tile => tile_properties.push(tile),
            };
        }

        Ok(tile_properties)
    }

    fn spawners_from_lparse(cnms: &LParse, version: &VersionSpecs, ignore_warnings: bool) -> Result<Vec<cnms_types::Spawner>, Error> {
        let mut spawners = Vec::new();

        for index in 0..cnms.try_get_entry("NUM_SPAWNERS")?.try_get_i32()?[0] as usize {
            spawners.push(cnms_types::Spawner::from_lparse(cnms, version, index, ignore_warnings)?);
        }

        Ok(spawners)
    }

    fn background_layers_from_lparse(cnmb: &LParse, version: &VersionSpecs) -> Result<Vec<BackgroundLayer>, Error> {
        let mut background_layers = Vec::new();

        for index in 0..version.background_layers {
            background_layers.push(cnmb_types::BackgroundLayer::from_lparse(cnmb, version, index)?);
        }

        Ok(background_layers)
    }
}
