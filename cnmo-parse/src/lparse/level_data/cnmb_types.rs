use bitflags::bitflags;

use crate::lparse::{EntryData, Error, LParse};

use crate::Rect;

use super::{consts::*, Duration, Point, VersionSpecs};

/// Background image represents the 2 types of background layers in CNM Online.
/// - Color: Clears the screen with the color pallete ID
/// - Bitmap: Represents an source rect from GFX.BMP
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum BackgroundImage {
    /// Clears the screen
    Color(u8),
    /// Uses an image from GFX.BMP
    Bitmap(Rect),
}

impl Default for BackgroundImage {
    fn default() -> Self {
        Self::Bitmap(Rect {
            x: 0,
            y: 0,
            w: 0,
            h: 0,
        })
    }
}

bitflags! {
    /// Flags used by background layers
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct BackgroundFlags: u32 {
        /// Specifies that this layer shows up on 4:3 displays
        const ShowOn4By3 = 1 << 0;
        /// Specifies that this layer shows up on 16:9 widescreen displays
        const ShowOn16By9 = 1 << 1;
    }
}

impl Default for BackgroundFlags {
    fn default() -> Self {
        BackgroundFlags::ShowOn4By3 | BackgroundFlags::ShowOn16By9
    }
}

/// A background layer in CNM Online
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Default, Clone)]
pub struct BackgroundLayer {
    /// Where the original image is (offset)
    pub origin: Point,
    /// Higher values equal less movement (looks like they are further away)
    /// a scroll value of 0 designates that there should be no scrolling.
    pub scroll_speed: Point,
    /// How fast the background scrolls in pixels per frame
    pub speed: Point,
    /// How far apart each repeating image should be in pixels.
    pub spacing: (i32, i32),
    /// The image style of the background
    pub image: BackgroundImage,

    /// Transparency of the background layer.
    pub transparency: u8,
    /// Should it repeat upwards
    pub repeat_up: bool,
    /// repeat downwards
    pub repeat_down: bool,
    /// Repeat both left and right
    pub repeat_horizontally: bool,
    /// Is the background drawn over all other game elements besides the HUD?
    pub in_foreground: bool,
    /// How big the bottom of the 3d projection is
    pub bottom3d: u32,
    /// How big the top of the 3d projection is
    pub top3d: u32,
    /// How high the 3d projection is
    pub height3d: u32,
    /// Background flags
    pub flags: BackgroundFlags,
}

impl BackgroundLayer {
    pub(crate) fn from_lparse(
        cnmb: &LParse,
        _version: &VersionSpecs,
        index: usize,
    ) -> Result<Self, Error> {
        let background_origin =
            &cnmb.try_get_entry("BG_ORIGIN")?.try_get_f32()?[index * 2..index * 2 + 2];
        let background_scroll =
            &cnmb.try_get_entry("BG_SCROLL")?.try_get_f32()?[index * 2..index * 2 + 2];
        let background_spacing =
            &cnmb.try_get_entry("BG_SPACING")?.try_get_i32()?[index * 2..index * 2 + 2];
        let background_speed =
            &cnmb.try_get_entry("BG_SPEED")?.try_get_f32()?[index * 2..index * 2 + 2];
        let background_repeat =
            &cnmb.try_get_entry("BG_REPEAT")?.try_get_u8()?[index * 2..index * 2 + 2];
        let background_rect = cnmb.try_get_entry("BG_RECT")?.try_get_rect()?[index];
        let background_clear_color = cnmb.try_get_entry("BG_CLEAR_COLOR")?.try_get_i32()?[index];
        let background_foreground = cnmb.try_get_entry("BG_HIGHLAYER")?.try_get_u8()?[index];
        let background_transparency = cnmb.try_get_entry("BG_TRANS")?.try_get_u8()?[index];
        let (background_top3d,
             background_bottom3d,
             background_height3d) = if let Ok(ratios) = cnmb.try_get_entry("BG_RATIO3D") {
            (
                ratios.try_get_i32()?[index * 3 + 0] as u32,
                ratios.try_get_i32()?[index * 3 + 1] as u32,
                ratios.try_get_i32()?[index * 3 + 2] as u32,
            )
        } else {
            (0, 0, 0)
        };
        let background_flags = if let Ok(flags) = cnmb.try_get_entry("BG_FLAGS") {
            BackgroundFlags::from_bits(flags.try_get_i32()?[index] as u32).unwrap_or(Default::default())
        } else {
            Default::default()
        };

        let image = if background_clear_color == 0 {
            BackgroundImage::Bitmap(background_rect)
        } else {
            BackgroundImage::Color(background_clear_color as u8)
        };

        Ok(Self {
            origin: Point(background_origin[0], background_origin[1]),
            scroll_speed: Point(background_scroll[0], background_scroll[1]),
            speed: Point(background_speed[0], background_speed[1]),
            spacing: (background_spacing[0], background_spacing[1]),
            image,
            transparency: background_transparency,
            repeat_up: background_repeat[1] & 2 != 0,
            repeat_down: background_repeat[1] & 1 != 0,
            repeat_horizontally: background_repeat[0] != 0,
            in_foreground: background_foreground != 0,
            top3d: background_top3d,
            bottom3d: background_bottom3d,
            height3d: background_height3d,
            flags: background_flags,
        })
    }

    pub(crate) fn save(
        &self,
        bg_origin: &mut Vec<f32>,
        bg_scroll: &mut Vec<f32>,
        bg_spacing: &mut Vec<i32>,
        bg_speed: &mut Vec<f32>,
        bg_repeat: &mut Vec<u8>,
        bg_rect: &mut Vec<Rect>,
        bg_clear_color: &mut Vec<i32>,
        bg_highlayer: &mut Vec<u8>,
        bg_trans: &mut Vec<u8>,
        bg_3d: &mut Vec<i32>,
        bg_flags: &mut Vec<i32>,
        _version: &VersionSpecs,
    ) {
        bg_origin.push(self.origin.0);
        bg_origin.push(self.origin.1);
        bg_scroll.push(self.scroll_speed.0);
        bg_scroll.push(self.scroll_speed.1);
        bg_spacing.push(self.spacing.0);
        bg_spacing.push(self.spacing.1);
        bg_speed.push(self.speed.0);
        bg_speed.push(self.speed.1);
        bg_repeat.push(self.repeat_horizontally as u8 * 3);
        bg_repeat.push((self.repeat_up as u8 * 2) | self.repeat_down as u8);
        match self.image {
            BackgroundImage::Bitmap(rect) => {
                bg_rect.push(rect);
                bg_clear_color.push(0);
            }
            BackgroundImage::Color(color) => {
                bg_rect.push(Rect {
                    x: 0,
                    y: 0,
                    w: 0,
                    h: 0,
                });
                bg_clear_color.push(color as i32);
            }
        };
        bg_highlayer.push(self.in_foreground as u8);
        bg_trans.push(self.transparency);
        bg_3d.push(self.top3d as i32);
        bg_3d.push(self.bottom3d as i32);
        bg_3d.push(self.height3d as i32);
        bg_flags.push((self.flags.0.bits() as u32) as i32);
    }
}

pub(super) fn save_background_vec(
    cnmb: &mut LParse,
    version: &VersionSpecs,
    backgrounds: &[BackgroundLayer],
) {
    let mut bg_pos = Vec::new();
    let mut bg_origin = Vec::new();
    let mut bg_scroll = Vec::new();
    let mut bg_spacing = Vec::new();
    let mut bg_speed = Vec::new();
    let mut bg_repeat = Vec::new();
    let mut bg_rect = Vec::new();
    let mut bg_clear_color = Vec::new();
    let mut bg_highlayer = Vec::new();
    let mut bg_trans = Vec::new();
    let mut bg_3d = Vec::new();
    let mut bg_flags = Vec::new();

    for background in &backgrounds[0..backgrounds.len().min(version.background_layers)] {
        bg_pos.push(0.0);
        bg_pos.push(0.0);
        background.save(
            &mut bg_origin,
            &mut bg_scroll,
            &mut bg_spacing,
            &mut bg_speed,
            &mut bg_repeat,
            &mut bg_rect,
            &mut bg_clear_color,
            &mut bg_highlayer,
            &mut bg_trans,
            &mut bg_3d,
            &mut bg_flags,
            version,
        );
    }

    cnmb.entries
        .insert("BG_POS".to_string(), EntryData::F32(bg_pos));
    cnmb.entries
        .insert("BG_ORIGIN".to_string(), EntryData::F32(bg_origin));
    cnmb.entries
        .insert("BG_SCROLL".to_string(), EntryData::F32(bg_scroll));
    cnmb.entries
        .insert("BG_SPACING".to_string(), EntryData::I32(bg_spacing));
    cnmb.entries
        .insert("BG_SPEED".to_string(), EntryData::F32(bg_speed));
    cnmb.entries
        .insert("BG_REPEAT".to_string(), EntryData::U8(bg_repeat));
    cnmb.entries
        .insert("BG_RECT".to_string(), EntryData::Rect(bg_rect));
    cnmb.entries
        .insert("BG_CLEAR_COLOR".to_string(), EntryData::I32(bg_clear_color));
    cnmb.entries
        .insert("BG_HIGHLAYER".to_string(), EntryData::U8(bg_highlayer));
    cnmb.entries
        .insert("BG_TRANS".to_string(), EntryData::U8(bg_trans));
    cnmb.entries
        .insert("BG_RATIO3D".to_string(), EntryData::I32(bg_3d));
    cnmb.entries
        .insert("BG_FLAGS".to_string(), EntryData::I32(bg_flags));
}

/// How a tile will damage the player
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub enum DamageType {
    /// It won't damage the player
    None,
    /// It will damage the player and act as a liquid
    Lava(i32),
    /// It will damage the player (no special effects)
    Spikes(i32),
    /// It will damage the player and act as quicksand
    Quicksand(i32),
    /// It will make the player's friction worse
    Ice(f32),
}

/// What collision type does the tile have?
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub enum CollisionType {
    /// A normal box
    Box(Rect),
    /// A Jumpthrough box
    Jumpthrough(Rect),
    /// A hightmap that has 0 at the bottom of the cell, and 32 at the top (i think).
    Heightmap([u8; TILE_SIZE]),
}

/// All properties of a tile.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct TileProperties {
    /// Is it solid
    pub solid: bool,
    /// Transparency level
    pub transparency: u8,
    /// Damage type
    pub damage_type: DamageType,
    /// How many frames until the tile changes its frame?
    pub anim_speed: Duration,
    /// "Angle" of the ground
    pub angle: u16,
    /// Tile locations in GFX.BMP of repsective frames (be careful of how many frames
    /// you can have, you need to look at [`crate::lparse::level_data::VersionSpecs`])
    pub frames: Vec<(i32, i32)>,
    /// How the block is shaped
    pub collision_data: CollisionType,
}

impl Default for TileProperties {
    fn default() -> Self {
        Self {
            solid: false,
            transparency: 0,
            damage_type: DamageType::None,
            anim_speed: Duration(1),
            frames: vec![(1, 0)],
            angle: 0,
            collision_data: CollisionType::Box(Rect {
                x: 0,
                y: 0,
                w: 32,
                h: 32,
            }),
        }
    }
}

impl TileProperties {
    pub(crate) fn from_lparse(
        cnmb: &LParse,
        version: &VersionSpecs,
        index: usize,
        ignore_warnings: bool,
    ) -> Result<Self, Error> {
        let block_flags = cnmb.try_get_entry("BP_FLAGS")?.try_get_u32()?;
        let block_transparency = cnmb.try_get_entry("BP_TRANS")?.try_get_i32()?;
        let block_damage_type = cnmb.try_get_entry("BP_DMG_TYPE")?.try_get_i32()?;
        let block_damage = cnmb.try_get_entry("BP_DMG")?.try_get_i32()?;
        let block_anim_speed = cnmb.try_get_entry("BP_ANIM_SPEED")?.try_get_i32()?;
        let block_num_frames = cnmb.try_get_entry("BP_NUM_FRAMES")?.try_get_i32()?;
        let block_frames_x = cnmb.try_get_entry("BP_FRAMESX")?.try_get_i32()?;
        let block_frames_y = cnmb.try_get_entry("BP_FRAMESY")?.try_get_i32()?;
        let block_heightmap = cnmb.try_get_entry("BP_HEIGHTMAP")?.try_get_u8()?;
        let block_hitbox = cnmb.try_get_entry("BP_HITBOX")?.try_get_rect()?;
        let block_collision_type = cnmb.try_get_entry("BP_COLLTYPE")?.try_get_i32()?;

        let solid = block_flags[index] & 1 != 0;
        let transparency = match block_transparency[index] {
            t if (t < LIGHT_WHITE as i32 || t > LIGHT_BLACK as i32) && !ignore_warnings => {
                return Err(Error::Corrupted(format!(
                    "Light out of the normal bounds of {LIGHT_BLACK} to {LIGHT_WHITE}"
                )))
            }
            t => t as u8,
        };
        let damage_type = match (block_damage_type[index], block_damage[index]) {
            (0, _) => DamageType::None,
            (1, damage) => DamageType::Lava(damage),
            (2, damage) => DamageType::Spikes(damage),
            (3, damage) => DamageType::Quicksand(damage),
            (4, damage) => DamageType::Ice(damage as f32 / 100.0),
            _ if !ignore_warnings => {
                return Err(Error::Corrupted("Unknown tile damage type!".to_string()))
            }
            _ => DamageType::None,
        };
        let anim_speed = Duration(block_anim_speed[index]);
        let frames = (0..(block_num_frames[index] & 0xff) as usize)
            .map(|frame| {
                (
                    block_frames_x[index * version.max_tile_frames + frame],
                    block_frames_y[index * version.max_tile_frames + frame],
                )
            })
            .collect();
        let angle = block_num_frames[index] >> 8;
        let collision_data = match block_collision_type[index] {
            0 => CollisionType::Box(block_hitbox[index]),
            2 => CollisionType::Jumpthrough(block_hitbox[index]),
            1 => {
                let mut heightmap = [0u8; TILE_SIZE];

                for pixel in 0..TILE_SIZE {
                    heightmap[pixel] = block_heightmap[index * TILE_SIZE + pixel];
                }

                CollisionType::Heightmap(heightmap)
            }
            _ if !ignore_warnings => {
                return Err(Error::Corrupted("Unknown tile collision type!".to_string()))
            }
            _ => CollisionType::Box(Rect {
                x: 0,
                y: 0,
                w: TILE_SIZE as i32,
                h: TILE_SIZE as i32,
            }),
        };

        Ok(TileProperties {
            solid,
            transparency,
            damage_type,
            anim_speed,
            angle: angle.try_into().unwrap(),
            frames,
            collision_data,
        })
    }

    fn save(
        &self,
        bp_flags: &mut Vec<u32>,
        bp_trans: &mut Vec<i32>,
        bp_dmg_type: &mut Vec<i32>,
        bp_dmg: &mut Vec<i32>,
        bp_anim_speed: &mut Vec<i32>,
        bp_num_frames: &mut Vec<i32>,
        bp_framesx: &mut Vec<i32>,
        bp_framesy: &mut Vec<i32>,
        bp_heightmap: &mut Vec<u8>,
        bp_hitbox: &mut Vec<Rect>,
        bp_colltype: &mut Vec<i32>,
        version: &VersionSpecs,
    ) {
        bp_flags.push(self.solid as u32);
        bp_trans.push(self.transparency as i32);
        let (dmg_type, dmg) = match self.damage_type {
            DamageType::None => (0, 0),
            DamageType::Lava(dmg) => (1, dmg),
            DamageType::Spikes(dmg) => (2, dmg),
            DamageType::Quicksand(dmg) => (3, dmg),
            DamageType::Ice(friction) => (4, (friction * 100.0) as i32),
        };
        bp_dmg_type.push(dmg_type);
        bp_dmg.push(dmg);
        bp_anim_speed.push(self.anim_speed.0);
        bp_num_frames.push(((self.frames.len() as i32) & 0xff) | ((self.angle as i32) << 8));
        let mut positions = self
            .frames
            .iter()
            .map(|frame| frame.0)
            .collect::<Vec<i32>>();
        positions.resize(version.max_tile_frames, 0);
        bp_framesx.append(&mut positions);
        positions = self
            .frames
            .iter()
            .map(|frame| frame.1)
            .collect::<Vec<i32>>();
        positions.resize(version.max_tile_frames, 0);
        bp_framesy.append(&mut positions);
        match self.collision_data {
            CollisionType::Box(rect) => {
                bp_colltype.push(0);
                bp_hitbox.push(rect);
                bp_heightmap.append(&mut vec![0].repeat(TILE_SIZE));
            }
            CollisionType::Jumpthrough(rect) => {
                bp_colltype.push(2);
                bp_hitbox.push(rect);
                bp_heightmap.append(&mut vec![0].repeat(TILE_SIZE));
            }
            CollisionType::Heightmap(heightmap) => {
                bp_colltype.push(1);
                bp_hitbox.push(Rect {
                    x: 0,
                    y: 0,
                    w: 0,
                    h: 0,
                });
                bp_heightmap.append(&mut heightmap.to_vec());
            }
        };
    }
}

pub(super) fn save_tile_properties_vec(
    cnmb: &mut LParse,
    version: &VersionSpecs,
    metadata_tile: &TileProperties,
    tile_properties: &[TileProperties],
) {
    let mut bp_flags = Vec::new();
    let mut bp_trans = Vec::new();
    let mut bp_dmg_type = Vec::new();
    let mut bp_dmg = Vec::new();
    let mut bp_anim_speed = Vec::new();
    let mut bp_num_frames = Vec::new();
    let mut bp_framesx = Vec::new();
    let mut bp_framesy = Vec::new();
    let mut bp_heightmap = Vec::new();
    let mut bp_hitbox = Vec::new();
    let mut bp_colltype = Vec::new();

    let air_tile = &TileProperties {
        frames: vec![(0, 0)],
        ..Default::default()
    };

    for idx in 0..(tile_properties.len() + 1).max(version.preview_tile_index) + 1 {
        let tile = match idx {
            idx if idx == 0
                || (idx != version.preview_tile_index && idx > tile_properties.len()) =>
            {
                &air_tile
            }
            idx if idx == version.preview_tile_index => metadata_tile,
            idx if idx < version.preview_tile_index => &tile_properties[idx - 1],
            idx if idx > version.preview_tile_index => &tile_properties[idx - 2],
            _ => &air_tile,
        };
        tile.save(
            &mut bp_flags,
            &mut bp_trans,
            &mut bp_dmg_type,
            &mut bp_dmg,
            &mut bp_anim_speed,
            &mut bp_num_frames,
            &mut bp_framesx,
            &mut bp_framesy,
            &mut bp_heightmap,
            &mut bp_hitbox,
            &mut bp_colltype,
            version,
        );
    }

    cnmb.entries
        .insert("BP_FLAGS".to_string(), EntryData::U32(bp_flags));
    cnmb.entries
        .insert("BP_TRANS".to_string(), EntryData::I32(bp_trans));
    cnmb.entries
        .insert("BP_DMG_TYPE".to_string(), EntryData::I32(bp_dmg_type));
    cnmb.entries
        .insert("BP_DMG".to_string(), EntryData::I32(bp_dmg));
    cnmb.entries
        .insert("BP_ANIM_SPEED".to_string(), EntryData::I32(bp_anim_speed));
    cnmb.entries
        .insert("BP_NUM_FRAMES".to_string(), EntryData::I32(bp_num_frames));
    cnmb.entries
        .insert("BP_FRAMESX".to_string(), EntryData::I32(bp_framesx));
    cnmb.entries
        .insert("BP_FRAMESY".to_string(), EntryData::I32(bp_framesy));
    cnmb.entries
        .insert("BP_HEIGHTMAP".to_string(), EntryData::U8(bp_heightmap));
    cnmb.entries
        .insert("BP_HITBOX".to_string(), EntryData::Rect(bp_hitbox));
    cnmb.entries
        .insert("BP_COLLTYPE".to_string(), EntryData::I32(bp_colltype));
}

/// Represents a tile in CNM Online
/// It is either air, or a refrence to the tile properties
/// array in [`crate::lparse::level_data::LevelData`]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Default, Copy, Clone)]
pub struct TileId(pub Option<u16>);

impl PartialEq for TileId {
    fn eq(&self, other: &Self) -> bool {
        match (self.0, other.0) {
            (Some(my_id), Some(other_id)) => my_id == other_id,
            (None, None) => true,
            _ => false,
        }
    }
}
impl Eq for TileId {}

impl TileId {
    /// Gets a safe [`TileId`]  from a raw CNM tile ID
    pub fn from_raw_id(raw: u16, version: &VersionSpecs) -> Self {
        if raw == 0 {
            Self(None)
        } else {
            if raw as usize > version.preview_tile_index {
                Self(Some(raw - 2))
            } else {
                Self(Some(raw - 1))
            }
        }
    }

    /// Returns the unsafe CNM raw tile ID
    pub fn get_raw_id(&self, version: &VersionSpecs) -> u16 {
        if let Some(id) = self.0 {
            if id as usize > version.preview_tile_index {
                id + 2
            } else {
                id + 1
            }
        } else {
            0
        }
    }
}

/// A cell on the cnm world grid.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Copy, Clone)]
pub struct Cell {
    /// Shown infront of cnm objects
    pub foreground: TileId,
    /// Shown behind cnm objects
    pub background: TileId,
    /// The light level of objects and both tiles on this grid point
    /// LIGHT_NORMAL should is no lighting effects
    pub light: u8,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            foreground: Default::default(),
            background: Default::default(),
            light: super::consts::LIGHT_NORMAL,
        }
    }
}

/// The grid of cells of a cnm world
#[derive(Debug, Clone)]
pub struct Cells {
    cells: Vec<Cell>,
    width: usize,
    height: usize,
}

#[cfg(feature = "serde")]
impl serde::Serialize for Cells {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut bytes = bytebuffer::ByteBuffer::new();
        bytes.set_endian(bytebuffer::Endian::BigEndian);
        bytes.write_u32(self.width as u32);
        bytes.write_u32(self.height as u32);
        for cell in self.cells.iter() {
            bytes.write_u16(
                cell.background
                    .get_raw_id(&VersionSpecs::from_version(1).unwrap()),
            );
            bytes.write_u16(
                cell.foreground
                    .get_raw_id(&VersionSpecs::from_version(1).unwrap()),
            );
            bytes.write_u8(cell.light);
        }

        serializer.serialize_str(base64::encode(bytes.as_bytes()).as_str())
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::de::Deserialize<'de> for Cells {
    fn deserialize<D: serde::de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_str(CellsVisitor)
    }
}

#[cfg(feature = "serde")]
struct CellsVisitor;

#[cfg(feature = "serde")]
impl<'de> serde::de::Visitor<'de> for CellsVisitor {
    type Value = Cells;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a base64 encoded array!")
    }

    fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
        let mut bytes = bytebuffer::ByteBuffer::from_vec(match base64::decode(v) {
            Ok(bytes) => bytes,
            Err(_) => return Err(serde::de::Error::custom("Inavlid base64!")),
        });
        bytes.set_endian(bytebuffer::Endian::BigEndian);

        let (width, height) = match (bytes.read_u32(), bytes.read_u32()) {
            (Ok(width), Ok(height)) => (width as usize, height as usize),
            _ => return Err(serde::de::Error::custom("Invalid bytes representation!")),
        };
        let mut cells = Vec::new();
        for _ in 0..width * height {
            let (background, foreground, light) =
                match (bytes.read_u16(), bytes.read_u16(), bytes.read_u8()) {
                    (Ok(background), Ok(foreground), Ok(light)) => (
                        TileId::from_raw_id(background, &VersionSpecs::from_version(1).unwrap()),
                        TileId::from_raw_id(foreground, &VersionSpecs::from_version(1).unwrap()),
                        light,
                    ),
                    _ => {
                        return Err(serde::de::Error::custom(
                            "Width and height don't match number of cells!",
                        ))
                    }
                };
            cells.push(Cell {
                background,
                foreground,
                light,
            });
        }

        Ok(Cells {
            cells,
            width,
            height,
        })
    }
}

impl Cells {
    /// Creates an empty grid of cells with the specified width and height
    pub fn new(width: usize, height: usize) -> Self {
        let cells = (0..width * height).map(|_| Cell::default()).collect();

        Self {
            width,
            height,
            cells,
        }
    }

    pub(crate) fn from_lparse(cnmb: &LParse, num_tile_properties: usize) -> Result<Self, Error> {
        let block_header = cnmb.try_get_entry("BLOCKS_HEADER")?.try_get_i32()?;
        let width = block_header[0] as usize;
        let height = block_header[1] as usize;

        let foreground_layer = cnmb.try_get_entry("BLK_LAYER0")?.try_get_u16()?;
        let background_layer = cnmb.try_get_entry("BLK_LAYER1")?.try_get_u16()?;
        let light_layer = cnmb.try_get_entry("BLK_LIGHT")?.try_get_u16()?;

        let cells = (0..width * height)
            .map(|index| {
                let mut background = TileId::from_raw_id(
                    background_layer[index],
                    &VersionSpecs::from_version(1).unwrap(),
                );
                let mut foreground = TileId::from_raw_id(
                    foreground_layer[index],
                    &VersionSpecs::from_version(1).unwrap(),
                );
                if let Some(id) = background.0 {
                    if id as usize >= num_tile_properties {
                        background.0 = None;
                    }
                }
                if let Some(id) = foreground.0 {
                    if id as usize >= num_tile_properties {
                        foreground.0 = None;
                    }
                }
                Cell {
                    background,
                    foreground,
                    light: u8::try_from(light_layer[index]).unwrap_or(LIGHT_NORMAL),
                }
            })
            .collect();

        Ok(Self {
            width,
            height,
            cells,
        })
    }

    pub(super) fn save(
        &self,
        cnmb: &mut LParse,
        num_tile_properties: usize,
        version: &VersionSpecs,
    ) {
        let mut blocks_header = Vec::new();
        let mut blk_layer0 = Vec::new();
        let mut blk_layer1 = Vec::new();
        let mut blk_light = Vec::new();

        blocks_header.push(self.width as i32);
        blocks_header.push(self.height as i32);
        blocks_header.push((num_tile_properties.max(version.preview_tile_index) + 1) as i32);

        for cell in self.cells.iter() {
            blk_layer0.push(cell.foreground.get_raw_id(version));
            blk_layer1.push(cell.background.get_raw_id(version));
            blk_light.push(cell.light as u16);
        }

        cnmb.entries
            .insert("BLOCKS_HEADER".to_string(), EntryData::I32(blocks_header));
        cnmb.entries
            .insert("BLK_LAYER0".to_string(), EntryData::U16(blk_layer0));
        cnmb.entries
            .insert("BLK_LAYER1".to_string(), EntryData::U16(blk_layer1));
        cnmb.entries
            .insert("BLK_LIGHT".to_string(), EntryData::U16(blk_light));
    }

    /// Returns a slice of the worlds cells
    pub fn cells(&self) -> &[Cell] {
        &self.cells
    }

    /// Returns a mutable slice of the world's cells
    pub fn cells_mut(&mut self) -> &mut [Cell] {
        &mut self.cells
    }

    /// Returns a cell ref from the world. The x and y positions are clamped
    /// to the world's borders.
    pub fn get_cell(&self, x: i32, y: i32) -> &Cell {
        let index = y.clamp(0, self.height as i32 - 1) * self.width as i32
            + x.clamp(0, self.width as i32 - 1);
        &self.cells[index as usize]
    }

    /// Returns a mut cell ref from the world. The x and y positions are clamped
    /// to the world's borders.
    pub fn get_cell_mut(&mut self, x: i32, y: i32) -> &mut Cell {
        let index = y.clamp(0, self.height as i32 - 1) * self.width as i32
            + x.clamp(0, self.width as i32 - 1);
        &mut self.cells[index as usize]
    }

    /// Width of the world in cells
    pub fn width(&self) -> usize {
        self.width
    }

    /// Height of the world in cells
    pub fn height(&self) -> usize {
        self.height
    }

    /// Resizes the world and keeps the old cells
    pub fn resize(&mut self, new_width: usize, new_height: usize) {
        if new_width == self.width && new_height == self.height {
            return;
        }

        let mut new_cells = Self::new(new_width, new_height);
        self.paste(
            &mut new_cells,
            (0, 0),
            (new_width as i32 - 1, new_height as i32 - 1),
            (0, 0),
        );
        *self = new_cells;
    }

    /// Pastes this cells grid into <other>.
    pub fn paste(
        &self,
        other: &mut Self,
        src_min: (i32, i32),
        src_max: (i32, i32),
        dst: (i32, i32),
    ) {
        for y in src_min.1..=src_max.1 {
            for x in src_min.0..=src_max.0 {
                *other.get_cell_mut(x - src_min.0 + dst.0, y - src_min.1 + dst.1) =
                    *self.get_cell(x, y);
            }
        }
    }
}
