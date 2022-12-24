use std::collections::HashMap;

#[cfg(feature = "level_data")]
pub mod level_data;

use super::Rect;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Can't open the file!")]
    CantOpenFile { source: std::io::Error },
    #[error("The file isn't a CNM LParse file.")]
    NotLParseFile,
    #[error("Unknown LParse version ({0}).")]
    UnknownVersion(u32),
    #[error("LParse file corrupted possibly for reason {0}.")]
    Corrupted(String),
    #[error("Entry {0} (id {1}) is corrupted.")]
    EntryCorrupted(String, usize),
    #[error("Entry {0} (id {1}) has a unknown type id of {0}!")]
    UnknownEntryType(String, usize, u32),
    #[error("Can't save file! Version number {0} only allows up to {1} entries, but you're trying to save {2} entries!")]
    EntryOverflow(u32, usize, usize),
    #[error("The CNMB file's version id {0} is mismatched with the CNMS file's version id {1}.")]
    MismatchedVersions(u32, u32),
    #[error("Can't find the entry {0}!")]
    CannotFindEntry(String),
    #[error("Unexpected entry data type! Expected type {0} but got type {1} instead!")]
    UnexpectedEntryType(u32, u32),
}

macro_rules! generate_entry_data_func {
    ($func_name:ident, $variant:ident, $return:ident) => {
        pub fn $func_name(&self) -> Result<&[$return], Error> {
            match self {
                &Self::$variant(ref vec) => Ok(vec),
                _ => Err(Error::UnexpectedEntryType(Self::$variant(Vec::new()).get_typeid(), self.get_typeid())),
            }
        }
    };
}

macro_rules! generate_entry_data_func_mut {
    ($func_name:ident, $variant:ident, $return:ident) => {
        pub fn $func_name(&mut self) -> Result<&mut Vec<$return>, Error> {
            match self {
                &mut Self::$variant(ref mut vec) => Ok(vec),
                _ => Err(Error::UnexpectedEntryType(Self::$variant(Vec::new()).get_typeid(), self.get_typeid())),
            }
        }
    };
}

#[derive(Debug)]
pub enum EntryData {
    Null,
    Dummy,
    I32(Vec<i32>),
    U32(Vec<u32>),
    U8(Vec<u8>),
    U16(Vec<u16>),
    F32(Vec<f32>),
    Rect(Vec<Rect>),
}

impl EntryData {
    fn from_lparse(
        typeid: u32,
        len: usize,
        name: &str,
        entry_id: usize,
        buffer: &mut bytebuffer::ByteBuffer,
    ) -> Result<Self, Error> {
        match typeid {
            0 => Ok(Self::Null),
            1 => Ok(Self::Dummy),
            2 => Ok(Self::I32(
                (0..len)
                    .map(|_| buffer.read_i32().unwrap_or_default())
                    .collect(),
            )),
            3 => Ok(Self::U32(
                (0..len)
                    .map(|_| buffer.read_u32().unwrap_or_default())
                    .collect(),
            )),
            4 => Ok(Self::U8(
                (0..len)
                    .map(|_| buffer.read_u8().unwrap_or_default())
                    .collect(),
            )),
            5 => Ok(Self::U16(
                (0..len)
                    .map(|_| buffer.read_u16().unwrap_or_default())
                    .collect(),
            )),
            6 => Ok(Self::F32(
                (0..len)
                    .map(|_| buffer.read_f32().unwrap_or_default())
                    .collect(),
            )),
            7 => Ok(Self::Rect(
                (0..len)
                    .map(|_| {
                        let x = buffer.read_i32().unwrap_or_default();
                        let y = buffer.read_i32().unwrap_or_default();
                        let w = buffer.read_i32().unwrap_or_default();
                        let h = buffer.read_i32().unwrap_or_default();
                        Rect { x, y, w, h }
                    })
                    .collect(),
            )),
            id => Err(Error::UnknownEntryType(name.to_string(), entry_id, id)),
        }
    }

    fn save(&self, buffer: &mut bytebuffer::ByteBuffer) {
        match &self {
            &Self::I32(vec) => vec.iter().for_each(|i| buffer.write_i32(*i)),
            &Self::U32(vec) => vec.iter().for_each(|i| buffer.write_u32(*i)),
            &Self::U8(vec) => vec.iter().for_each(|i| buffer.write_u8(*i)),
            &Self::U16(vec) => vec.iter().for_each(|i| buffer.write_u16(*i)),
            &Self::F32(vec) => vec.iter().for_each(|i| buffer.write_f32(*i)),
            &Self::Rect(vec) => vec.iter().for_each(|Rect { x, y, w, h }| {
                buffer.write_i32(*x);
                buffer.write_i32(*y);
                buffer.write_i32(*w);
                buffer.write_i32(*h);
            }),
            _ => {}
        }
    }

    fn get_entry_len(&self) -> usize {
        match &self {
            &Self::Null | &Self::Dummy => 0,
            &Self::I32(vec) => vec.len(),
            &Self::U32(vec) => vec.len(),
            &Self::U8(vec) => vec.len(),
            &Self::U16(vec) => vec.len(),
            &Self::F32(vec) => vec.len(),
            &Self::Rect(vec) => vec.len(),
        }
    }

    fn get_saved_data_size(&self) -> usize {
        match &self {
            &Self::Null | &Self::Dummy => 0,
            &Self::I32(vec) => vec.len() * std::mem::size_of::<i32>(),
            &Self::U32(vec) => vec.len() * std::mem::size_of::<u32>(),
            &Self::U8(vec) => vec.len() * std::mem::size_of::<u8>(),
            &Self::U16(vec) => vec.len() * std::mem::size_of::<u16>(),
            &Self::F32(vec) => vec.len() * std::mem::size_of::<f32>(),
            &Self::Rect(vec) => vec.len() * std::mem::size_of::<Rect>(),
        }
    }

    fn get_typeid(&self) -> u32 {
        match &self {
            &Self::Null => 0,
            &Self::Dummy => 1,
            &Self::I32(_) => 2,
            &Self::U32(_) => 3,
            &Self::U8(_) => 4,
            &Self::U16(_) => 5,
            &Self::F32(_) => 6,
            &Self::Rect(_) => 7,
        }
    }

    generate_entry_data_func!(try_get_i32, I32, i32);
    generate_entry_data_func!(try_get_u32, U32, u32);
    generate_entry_data_func!(try_get_u8, U8, u8);
    generate_entry_data_func!(try_get_u16, U16, u16);
    generate_entry_data_func!(try_get_f32, F32, f32);
    generate_entry_data_func!(try_get_rect, Rect, Rect);
    generate_entry_data_func_mut!(try_get_i32_mut, I32, i32);
    generate_entry_data_func_mut!(try_get_u32_mut, U32, u32);
    generate_entry_data_func_mut!(try_get_u8_mut, U8, u8);
    generate_entry_data_func_mut!(try_get_u16_mut, U16, u16);
    generate_entry_data_func_mut!(try_get_f32_mut, F32, f32);
    generate_entry_data_func_mut!(try_get_rect_mut, Rect, Rect);
}

#[derive(Debug)]
pub struct VersionSpecs {
    version: u32,
    num_entries: usize,
    entry_name_size: usize,
    header_size: usize,
    entry_header_size: usize,
}

impl VersionSpecs {
    pub fn from_version(version: u32) -> Result<Self, Error> {
        match version {
            1 => Ok(Self {
                version,
                num_entries: 128,
                entry_name_size: 16,
                header_size: 4 + std::mem::size_of::<u32>(),
                entry_header_size: std::mem::size_of::<u32>() * 3 + 16,
            }),
            version_id => Err(Error::UnknownVersion(version_id)),
        }
    }

    pub fn get_version_id(&self) -> u32 {
        self.version
    }

    pub fn get_num_entries(&self) -> usize {
        self.num_entries
    }

    pub fn get_entry_name_size(&self) -> usize {
        self.entry_name_size
    }

    pub fn get_header_size(&self) -> usize {
        self.header_size
    }

    pub fn get_entry_header_size(&self) -> usize {
        self.entry_header_size
    }
}

#[derive(Debug)]
pub struct LParse {
    version: VersionSpecs,
    pub entries: HashMap<String, EntryData>,
}

impl LParse {
    pub fn new(version: u32) -> Result<Self, Error> {
        Ok(Self {
            version: VersionSpecs::from_version(version)?,
            entries: HashMap::new(),
        })
    }

    pub fn get_version(&self) -> &VersionSpecs {
        &self.version
    }

    pub fn try_get_entry(&self, name: &str) -> Result<&EntryData, Error> {
        match self.entries.get(name) {
            Some(entry_data) => Ok(entry_data),
            None => Err(Error::CannotFindEntry(name.to_string())),
        }
    }

    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self, Error> {
        let buffer = match std::fs::read(path) {
            Ok(f) => f,
            Err(e) => return Err(Error::CantOpenFile { source: e }),
        };

        Self::from_memory(buffer)
    }

    pub fn from_memory(buffer: Vec<u8>) -> Result<Self, Error> {
        let mut buffer = bytebuffer::ByteBuffer::from_vec(buffer);
        buffer.set_endian(bytebuffer::Endian::LittleEndian);

        match buffer.read_bytes(4) {
            Ok(bytes) => match String::from_utf8(bytes) {
                Ok(s) if s == "CNML" => (),
                _ => return Err(Error::NotLParseFile),
            },
            Err(_) => return Err(Error::NotLParseFile),
        };

        let version = VersionSpecs::from_version(match buffer.read_u32() {
            Ok(x) => x,
            Err(_) => return Err(Error::NotLParseFile),
        })?;
        let mut entries = HashMap::new();

        for entry_id in 0..version.num_entries {
            let name = match buffer.read_bytes(version.entry_name_size) {
                Ok(b) => match String::from_utf8(b.clone()) {
                    Ok(s) => s.trim_end_matches('\0').to_string(),
                    Err(_) => {
                        return Err(Error::EntryCorrupted(
                            String::from_utf8_lossy(&b).into_owned(),
                            entry_id,
                        ))
                    }
                },
                Err(_) => {
                    return Err(Error::EntryCorrupted(
                        "<Entry name corrupted>".to_string(),
                        entry_id,
                    ))
                }
            };

            let (typeid, len, offset) =
                match (buffer.read_u32(), buffer.read_u32(), buffer.read_u32()) {
                    (Ok(typeid), Ok(len), Ok(offset)) => (typeid, len, offset),
                    _ => return Err(Error::EntryCorrupted(name.clone(), entry_id)),
                };

            // Add the new entry
            let mut data = EntryData::Null;
            if name.len() != 0
                && typeid != EntryData::Null.get_typeid()
                && typeid != EntryData::Dummy.get_typeid()
            {
                let loc = buffer.get_rpos();
                buffer.set_rpos(offset as usize);
                data = EntryData::from_lparse(typeid, len as usize, &name, entry_id, &mut buffer)?;
                buffer.set_rpos(loc);
            }

            if name.len() != 0 {
                entries.insert(name, data);
            }
        }

        Ok(Self { version, entries })
    }

    pub fn save_to_file<P: AsRef<std::path::Path>>(&self, path: P) -> Result<(), Error> {
        if self.entries.len() > self.version.num_entries {
            return Err(Error::EntryOverflow(self.version.version, self.version.num_entries, self.entries.len()))
        }

        let mut buffer = bytebuffer::ByteBuffer::new();
        buffer.set_endian(bytebuffer::Endian::LittleEndian);

        buffer.write_bytes(&"CNML".as_bytes()[0..4]);
        buffer.write_u32(self.version.version);

        let mut data_offset =
            self.version.header_size + self.version.entry_header_size * self.version.num_entries;
        let null_entry = (&"".to_string(), &EntryData::Null);
        for (name, data) in self
            .entries
            .iter()
            .chain((0..self.version.num_entries - self.entries.len()).map(|_| null_entry))
        {
            let mut name_padded = name.clone();
            name_padded.extend((0..self.version.entry_name_size).map(|_| '\0'));
            buffer.write_bytes(&name_padded.as_bytes()[0..self.version.entry_name_size]);
            buffer.write_u32(data.get_typeid());
            buffer.write_u32(data.get_entry_len() as u32);
            buffer.write_u32(data_offset as u32);
            data_offset += data.get_saved_data_size();
        }
        for (_, data) in self.entries.iter() {
            data.save(&mut buffer);
        }

        match std::fs::write(path, buffer.as_bytes()) {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::CantOpenFile { source: e }),
        }
    }
}
