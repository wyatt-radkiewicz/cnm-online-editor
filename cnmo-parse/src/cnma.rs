use std::fmt::Display;

/// Errors when dealing with CNMA files
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// [`from_file`](crate::cnma::Cnma::from_file) and [`save`](crate::cnma::Cnma::save) can return this
    #[error("Can't open the file!")]
    CantOpenFile {
        /// The actuall [`std::io::Error`] from reading/writing to the file.
        source: std::io::Error
    },
    /// The file is not a CNM config file.
    #[error("The file isn't a CNM Audio Definition file.")]
    NotCnmaFile,
    /// There was a corrupted entry at the line specified
    #[error("Cnma file has a corrupt entry at line {0}!")]
    CorruptedEntry(usize),
    /// There was an mode field but it had no mode name
    #[error("Cnma file has an entry without a mode!")]
    NoMode,
    /// The file is corrupted because of the string inside of the tuple variant
    #[error("Cnma file is corrupted because of {0}!")]
    Corrupted(String),
}

/// Used in the SoundID and the MusicID modes to specify what file a sound
/// is related to and its related ID.
#[derive(Debug, Clone, PartialEq)]
pub struct ResourceId {
    /// The ID for CNM Online. If 2 are the same, the last is used and a memory leak occurs.
    pub id: u32,
    /// The path to the file in question. It is relative based on the exe's directory and
    /// doesn't need a "./" at the start.
    pub path: String,
}

impl Display for ResourceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Id: {}, Path: {}", self.id, self.path))
    }
}

impl ResourceId {
    fn from_line(line_num: usize, line: &str) -> Result<Self, Error> {
        Ok(ResourceId {
            id: match line.split_whitespace().nth(0) {
                Some(id) => match id.parse() {
                    Ok(id) => id,
                    Err(_) => return Err(Error::CorruptedEntry(line_num + 1)),
                },
                None => return Err(Error::CorruptedEntry(line_num + 1)),
            },
            path: match line.split_whitespace().nth(1) {
                Some(path) => path.to_string(),
                None => return Err(Error::CorruptedEntry(line_num + 1)),
            }
        })
    }
}

/// A power defintion for a particular skin id, or any id.
/// These are activated when hitting the MaxPower trigger in game
/// and modify your player stats.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct MaxPowerDef {
    /// The skin id for this power definition
    pub id: u8,
    /// The speed multiplier
    pub speed: f32,
    /// Jump multiplier
    pub jump: f32,
    /// Gravity multiplier
    pub gravity: f32,
    /// How many hp points are removed per second while the power is active
    pub hpcost: f32,
    /// The strength multiplier
    pub strength: f32,
    /// An optional double jump ability.
    pub ability: Option<MaxPowerAbility>,
}

/// A ability activated on a double jump.
#[derive(Debug, Default, Clone, PartialEq)]
pub enum MaxPowerAbility {
    /// Basic double jump
    #[default]
    DoubleJump,
    /// Basic flying ability akin to the normal wings power up
    Flying,
    /// Activates a sheild once you hit the ground for a short moment
    /// of time
    DropShield,
    /// Allows you to bounce on enemies but not hurt them.
    MarioBounce,
}

/// What section/mode contents there are
#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    /// Music Resource Ids
    MusicIds(Vec<ResourceId>),
    /// Sound Resource Ids
    SoundIds(Vec<ResourceId>),
    /// A deprecated CNM Online section
    MusicVolumeOverride,
    /// The order of levels on the level select menu, same order
    /// as the strings in the vector here.
    LevelSelectOrder(Vec<String>),
    /// A power defintion for a particular skin
    MaxPowerDef(MaxPowerDef),
    /// Code run at the beginning of the game, certain function names
    /// will run as hooks for object code, etc.
    LuaAutorunCode(String),
}

/// CNMA file. Holds generic configuration of the game and resource
/// locations.
/// - Sound and music ids and file paths
/// - Lua scripting code
/// - Custom upgrade/powers
/// - The order of the level select menu
#[derive(Debug)]
pub struct Cnma {
    /// Vector of the sections of the file.
    pub modes: Vec<Mode>,
}

impl Cnma {
    /// Load a Cnma config from a file
    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self, Error> {
        let s = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => return Err(Error::CantOpenFile { source: e }),
        };
        Self::from_string(s.as_str())
    }

    /// Load a Cnma config from a string
    pub fn from_string(s: &str) -> Result<Self, Error> {
        let mut cnma = Cnma { modes: Vec::new() };
        let mut current_mode: Option<Mode> = None;
        let mut mode_locked = false;

        let append_mode = |cnma: &mut Cnma, current_mode: &mut Option<Mode>| {
            if let Some(mode) = current_mode.as_mut() {
                if let Mode::LevelSelectOrder(ref mut vec) = mode {
                    vec.reverse();
                }

                cnma.modes.push(mode.clone());
            }
        };
        
        for (line_num, line) in s.lines().enumerate() {
            if line.starts_with("MODE") && !mode_locked {
                append_mode(&mut cnma, &mut current_mode);
                current_mode = match line.split_whitespace().nth(1) {
                    Some("MUSIC") => Some(Mode::MusicIds(Vec::new())),
                    Some("SOUNDS") => Some(Mode::SoundIds(Vec::new())),
                    Some("MUSIC_VOLUME_OVERRIDE") => Some(Mode::MusicVolumeOverride),
                    Some("LEVELSELECT_ORDER") => Some(Mode::LevelSelectOrder(Vec::new())),
                    Some(s) if s.starts_with("MAXPOWER") => Some(Mode::MaxPowerDef(MaxPowerDef {
                        id: s[s.find(|c: char| c.is_digit(10)).unwrap_or_default()..s.len()].parse().unwrap_or_default(),
                        ..Default::default()
                    })),
                    Some("LUA_AUTORUN") => {
                        mode_locked = true;
                        Some(Mode::LuaAutorunCode("".to_string()))
                    },
                    Some(mode_name) => return Err(Error::Corrupted(format!("Unkown audio mode name {mode_name}"))),
                    None => return Err(Error::Corrupted(format!("Expected a mode name after \"MODE\" on line {}", line_num + 1))),
                };

                continue;
            } else if line.starts_with("__ENDLUA__") && mode_locked {
                match current_mode {
                    Some(Mode::LuaAutorunCode(_)) => {
                        mode_locked = false;
                        cnma.modes.push(current_mode.as_ref().unwrap().clone());
                        current_mode = None;
                    },
                    _ => return Err(Error::Corrupted("__ENDLUA__ found outside of LUA_AUTORUN mode segment!".to_string())),
                }

                continue;
            }

            match current_mode.as_mut() {
                Some(&mut Mode::MusicIds(ref mut v)) => {
                    v.push(ResourceId::from_line(line_num, line)?)
                },
                Some(&mut Mode::SoundIds(ref mut v)) => {
                    v.push(ResourceId::from_line(line_num, line)?)
                },
                Some(&mut Mode::MusicVolumeOverride) => {},
                Some(&mut Mode::LevelSelectOrder(ref mut v)) => {
                    match line.split_whitespace().nth(0) {
                        Some(s) => v.push(s.to_string()),
                        None => return Err(Error::CorruptedEntry(line_num + 1)),
                    }
                },
                Some(&mut Mode::MaxPowerDef(ref mut def)) => {
                    let (field_name, field_value) = match (line.split_whitespace().nth(0), line.split_whitespace().nth(1)) {
                        (Some(n), Some(v)) => (n, v),
                        _ => return Err(Error::CorruptedEntry(line_num + 1)),
                    };

                    match field_name {
                        "spd" => def.speed = field_value.parse().unwrap_or_default(),
                        "jmp" => def.jump = field_value.parse().unwrap_or_default(),
                        "grav" => def.gravity = field_value.parse().unwrap_or_default(),
                        "hpcost" => def.hpcost = field_value.parse().unwrap_or_default(),
                        "strength" => def.strength = field_value.parse().unwrap_or_default(),
                        "ability" => def.ability = match field_value.parse().unwrap_or_default() {
                            0 => None,
                            1 => Some(MaxPowerAbility::DoubleJump),
                            2 => Some(MaxPowerAbility::Flying),
                            3 => Some(MaxPowerAbility::DropShield),
                            4 => Some(MaxPowerAbility::MarioBounce),
                            _ => return Err(Error::CorruptedEntry(line_num + 1)),
                        },
                        _ => return Err(Error::CorruptedEntry(line_num + 1)),
                    };
                },
                Some(&mut Mode::LuaAutorunCode(ref mut code)) => {
                    code.push_str((line.to_string() + "\n").as_str());
                },
                None => return Err(Error::NoMode),
            }
        }

        append_mode(&mut cnma, &mut current_mode);

        Ok(cnma)
    }

    /// Saves the cnma file to the path specified, creates the file if it doesn't
    /// exist and overwrites it if it does.
    pub fn save<P: AsRef<std::path::Path>>(&self, path: P) -> Result<(), Error> {
        let mut contents = "".to_string();
        
        for mode in self.modes.iter() {
            match mode {
                &Mode::MusicIds(ref v) => {
                    contents.push_str("MODE MUSIC\n");
                    for res in v.iter() {
                        contents.push_str(format!("{} {}\n", res.id, res.path).as_str());
                    }
                },
                &Mode::SoundIds(ref v) => {
                    contents.push_str("MODE SOUNDS\n");
                    for res in v.iter() {
                        contents.push_str(format!("{} {}\n", res.id, res.path).as_str());
                    }
                },
                &Mode::MusicVolumeOverride => {
                    contents.push_str("MODE MUSIC_VOLUME_OVERRIDE\n");
                },
                &Mode::LevelSelectOrder(ref v) => {
                    contents.push_str("MODE LEVELSELECT_ORDER\n");
                    for lvl in v.iter().rev() {
                        contents.push_str(format!("{} _\n", lvl).as_str());
                    }
                },
                &Mode::MaxPowerDef(ref def) => {
                    let ability_id = match def.ability {
                        None => 0,
                        Some(MaxPowerAbility::DoubleJump) => 1,
                        Some(MaxPowerAbility::Flying) => 2,
                        Some(MaxPowerAbility::DropShield) => 3,
                        Some(MaxPowerAbility::MarioBounce) => 4,
                    };

                    contents.push_str(format!("MODE MAXPOWER{}\n", def.id).as_str());
                    contents.push_str(format!("spd {}\n", def.speed).as_str());
                    contents.push_str(format!("jmp {}\n", def.jump).as_str());
                    contents.push_str(format!("grav {}\n", def.gravity).as_str());
                    contents.push_str(format!("hpcost {}\n", def.hpcost).as_str());
                    contents.push_str(format!("strength {}\n", def.strength).as_str());
                    contents.push_str(format!("ability {}\n", ability_id).as_str());
                },
                &Mode::LuaAutorunCode(ref s) => {
                    contents.push_str("MODE LUA_AUTORUN\n");
                    contents.push_str(s.as_str());
                },
            }

            if let &Mode::LuaAutorunCode(_) = mode {
                contents.push_str("__ENDLUA__\n");
            }
        }

        match std::fs::write(path, contents) {
            Err(e) => Err(Error::CantOpenFile { source: e }),
            _ => Ok(()),
        }
    }
}
