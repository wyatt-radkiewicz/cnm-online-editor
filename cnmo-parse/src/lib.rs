#![warn(missing_docs)]
//! A all-encompasing CNM Online lparse and cnma data format parser.
//!
//! This crate allows one to load .cnmb/.cnms/.cnma files.
//! - .cnmb/.cnms files are binary .lparse files and start with the bytes "CNML"
//! - .cnma files are text files with a special format, kind of like .ini files
//! 
//! By default this crate only allows one to edit these files at the
//! very lowest level (besides for cnma files which can already load
//! in game configs), so if you want to load a cnm level file from an
//! lparse file, you will need to specify certain crate features.
//! 
//! These features are:
//! - "level_data" which adds in structs that represent all aspects
//!     of a CNM online level file, and has functions to save and load
//!     them from their respective .cnmb and .cnms lparse files.
//! - "serde" which adds serde traits for said level data structs
//!     so that the level data can also additionally be saved to any
//!     other format that you want. This is NOT a serde implementation
//!     for lparse or cnma files (and you need "level_data" feature
//!     enabled already).
//! 
//! Heres the [`Github Link`]
//! 
//! [`Github Link`]: https://github.com/wyatt-radkiewicz/cnm-online-editor/tree/main/cnmo-parse

/// Loading and saving lparse (level) files
pub mod lparse;
/// Loading and saving cnm config files
pub mod cnma;

/// A generic rectangle struct that CNM Online uses internally
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    /// X position of the rect
    pub x: i32,
    /// Y position of the rect
    pub y: i32,
    /// Width of the rect. Negative values are undefined and might crash CNM Online. These
    /// are i32's because in the CNM Online source code and lparse files they are i32's.
    pub w: i32,
    /// Height of the rect. Negative values are undefined and might crash CNM Online. These
    /// are i32's because in the CNM Online source code and lparse files they are i32's.
    pub h: i32,
}

impl Rect {
    /// Creates a new rectangle
    /// 
    /// # Example
    /// 
    /// ```
    /// # use cnmo_parse::Rect;
    /// let rect = Rect::new(0, 0, 100, 100);
    /// ```
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self { x, y, w, h }
    }
}

impl std::fmt::Display for Rect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("")
    }
}
