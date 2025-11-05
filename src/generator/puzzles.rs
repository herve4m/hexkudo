/*
puzzle.rs

Copyright 2025 Hervé Quatremain

This file is part of Hexkudo.

Hexkudo is free software: you can redistribute it and/or modify it under the
terms of the GNU General Public License as published by the Free Software
Foundation, either version 3 of the License, or (at your option) any later
version.

Hexkudo is distributed in the hope that it will be useful, but WITHOUT ANY
WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR
A PARTICULAR PURPOSE. See the GNU General Public License for more details.

You should have received a copy of the GNU General Public License along with
Hexkudo. If not, see <https://www.gnu.org/licenses/>.

SPDX-License-Identifier: GPL-3.0-or-later
*/

//! Puzzle internal representation

// For developers: add you new puzzle to this list of modules.
pub mod easy_classic_22;
pub mod easy_heart_24;
pub mod easy_square_22;
pub mod hard_classic_60;
pub mod hard_heart_58;
pub mod hard_square_60;
pub mod medium_classic_36;
pub mod medium_heart_45;
pub mod medium_square_38;

use super::puzzle_parse;
use clap::ValueEnum;
use gettextrs::gettext;
use gtk::glib;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use strum_macros::FromRepr;

/// Puzzle difficulty level.
#[derive(
    Serialize,
    Deserialize,
    Debug,
    Copy,
    Clone,
    PartialOrd,
    PartialEq,
    Eq,
    Hash,
    ValueEnum,
    FromRepr,
    Default,
    glib::Enum,
)]
#[repr(i32)]
#[enum_type(name = "Difficulty")]
pub enum Difficulty {
    #[default]
    Easy,
    Medium,
    Hard,
}

impl fmt::Display for Difficulty {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Difficulty::Easy => write!(f, "{}", gettext("Easy")),
            Difficulty::Medium => write!(f, "{}", gettext("Medium")),
            Difficulty::Hard => write!(f, "{}", gettext("Hard")),
        }
    }
}

/// Cell colors.
///
/// Color components are integers between 0 and 255.
#[derive(Debug, Clone)]
pub struct PuzzleColor {
    /// Cell borders.
    pub border: (u8, u8, u8, u8),

    /// Cell background.
    pub bg: (u8, u8, u8, u8),

    /// Background color for the cells with the initial hint.
    pub bg_map: (u8, u8, u8, u8),

    /// Number colors.
    pub text: (u8, u8, u8, u8),

    /// Diamonds colors.
    pub diamond: (u8, u8, u8, u8),

    /// Text color for cells with errors.
    pub text_wrong: (u8, u8, u8, u8),

    /// Background color of the selected cell.
    pub selection: (u8, u8, u8, u8),

    /// Path line over the puzzle (solution).
    pub path: (u8, u8, u8, u8),

    /// CSS string used for the puzzle background.
    /// If empty, then the default application background is used.
    pub bg_css: &'static str,
}

/// Custom colors set by the user.
///
/// Color components are floats between 0 and 1.
#[derive(Debug, Clone)]
pub struct PuzzleCustomColor {
    /// Cell borders and diamonds.
    border: (f64, f64, f64, f64),

    /// Whether to use this custom color or the default puzzle color for borders and diamonds.
    border_custom: bool,

    /// Cell background.
    bg: (f64, f64, f64, f64),

    /// Whether to use this custom color or the default puzzle color for cell background.
    bg_custom: bool,

    /// Background color for the cells with the initial hint (mapped cells).
    bg_map: (f64, f64, f64, f64),

    /// Whether to use this custom color or the default puzzle color for the background of mapped
    /// (or hint) cells.
    bg_map_custom: bool,

    /// Number colors.
    text: (f64, f64, f64, f64),

    /// Whether to use this custom color or the default puzzle color for numbers.
    text_custom: bool,

    /// Text color for cells with errors.
    text_wrong: (f64, f64, f64, f64),

    /// Whether to use this custom color or the default puzzle color for wrong values.
    text_wrong_custom: bool,

    /// Background color of the selected cell.
    selection: (f64, f64, f64, f64),

    /// Whether to use this custom color or the default puzzle color for the background of the
    /// selected cell.
    selection_custom: bool,

    /// Path line over the puzzle (solution).
    path: (f64, f64, f64, f64),

    /// Whether to use this custom color or the default puzzle color for the path.
    path_custom: bool,
}

impl Default for PuzzleCustomColor {
    fn default() -> Self {
        Self::new()
    }
}

impl PuzzleCustomColor {
    /// Create a [`PuzzleCustomColor`] object.
    pub fn new() -> Self {
        Self {
            border: (0.0, 0.0, 0.0, 1.0),
            border_custom: false,
            bg: (0.98, 0.98, 0.98, 1.0),
            bg_custom: false,
            bg_map: (0.894, 0.894, 0.894, 1.0),
            bg_map_custom: false,
            text: (0.0, 0.0, 0.0, 1.0),
            text_custom: false,
            text_wrong: (0.502, 0.0, 0.0, 1.0),
            text_wrong_custom: false,
            selection: (0.568, 0.737, 1.0, 1.0),
            selection_custom: false,
            path: (0.0, 0.0, 0.0, 0.376),
            path_custom: false,
        }
    }

    /// Set the borders and diamonds color.
    pub fn set_border(&mut self, red: f64, green: f64, blue: f64, alpha: f64) {
        self.border = (red, green, blue, alpha);
    }

    /// Set whether to use the custom or the default color for borders and diamonds.
    pub fn set_custom_border(&mut self, custom: bool) {
        self.border_custom = custom;
    }

    /// Return the custom color for the borders and diamonds, or None if the default color must
    /// be used.
    pub fn get_border(&self) -> Option<(f64, f64, f64, f64)> {
        if self.border_custom {
            return Some(self.border);
        }
        None
    }

    /// Set the cell background color.
    pub fn set_bg(&mut self, red: f64, green: f64, blue: f64, alpha: f64) {
        self.bg = (red, green, blue, alpha);
    }

    /// Set whether to use the custom or the default color for cell background.
    pub fn set_custom_bg(&mut self, custom: bool) {
        self.bg_custom = custom;
    }

    /// Return the custom color for the cell background, or None if the default color must
    /// be used.
    pub fn get_bg(&self) -> Option<(f64, f64, f64, f64)> {
        if self.bg_custom {
            return Some(self.bg);
        }
        None
    }

    /// Set the background color of mapped (hint) cells.
    pub fn set_bg_map(&mut self, red: f64, green: f64, blue: f64, alpha: f64) {
        self.bg_map = (red, green, blue, alpha);
    }

    /// Set whether to use the custom or the default color for mapped (hint) cells.
    pub fn set_custom_bg_map(&mut self, custom: bool) {
        self.bg_map_custom = custom;
    }

    /// Return the custom color for the mapped cell background, or None if the default color must
    /// be used.
    pub fn get_bg_map(&self) -> Option<(f64, f64, f64, f64)> {
        if self.bg_map_custom {
            return Some(self.bg_map);
        }
        None
    }

    /// Set the number color.
    pub fn set_text(&mut self, red: f64, green: f64, blue: f64, alpha: f64) {
        self.text = (red, green, blue, alpha);
    }

    /// Set whether to use the custom or the default color for the numbers.
    pub fn set_custom_text(&mut self, custom: bool) {
        self.text_custom = custom;
    }

    /// Return the custom color for the numbers, or None if the default color must be used.
    pub fn get_text(&self) -> Option<(f64, f64, f64, f64)> {
        if self.text_custom {
            return Some(self.text);
        }
        None
    }

    /// Set the color of the numbers for erroneous or duplicated cells.
    pub fn set_text_wrong(&mut self, red: f64, green: f64, blue: f64, alpha: f64) {
        self.text_wrong = (red, green, blue, alpha);
    }

    /// Set whether to use the custom or the default color for the numbers of erroneous cells.
    pub fn set_custom_text_wrong(&mut self, custom: bool) {
        self.text_wrong_custom = custom;
    }

    /// Return the custom color for the numbers of erroneous cells, or None if the default color
    /// must be used.
    pub fn get_text_wrong(&self) -> Option<(f64, f64, f64, f64)> {
        if self.text_wrong_custom {
            return Some(self.text_wrong);
        }
        None
    }

    /// Set the background color of the selected cell.
    pub fn set_selection(&mut self, red: f64, green: f64, blue: f64, alpha: f64) {
        self.selection = (red, green, blue, alpha);
    }

    /// Set whether to use the custom or the default color for the background of the selected cell.
    pub fn set_custom_selection(&mut self, custom: bool) {
        self.selection_custom = custom;
    }

    /// Return the custom color for the background of the selected cell, or None if the default
    /// color must be used.
    pub fn get_selection(&self) -> Option<(f64, f64, f64, f64)> {
        if self.selection_custom {
            return Some(self.selection);
        }
        None
    }

    /// Set the color of the path (the line over the cells).
    pub fn set_path(&mut self, red: f64, green: f64, blue: f64, alpha: f64) {
        self.path = (red, green, blue, alpha);
    }

    /// Set whether to use the custom or the default color for the path (the line over the cells).
    pub fn set_custom_path(&mut self, custom: bool) {
        self.path_custom = custom;
    }

    /// Return the custom color for the path, or None if the default color must be used.
    pub fn get_path(&self) -> Option<(f64, f64, f64, f64)> {
        if self.path_custom {
            return Some(self.path);
        }
        None
    }
}

/// Manage the colors for the puzzle.
#[derive(Debug, Clone)]
pub struct PuzzleColorTheme {
    /// Colors for the light theme.
    light: PuzzleColor,

    /// Colors for the dark theme.
    dark: PuzzleColor,

    /// Colors set by the user. These colors overwrite the default colors in `light` and `dark`.
    pub custom: PuzzleCustomColor,

    /// Whether ti use the dark or the light theme.
    is_dark: bool,
}

impl PuzzleColorTheme {
    /// Switch to the dark theme.
    pub fn set_dark(&mut self, is_dark: bool) {
        self.is_dark = is_dark;
    }

    /// Convert a color in the 0-255 range to the 0-1 range.
    fn to_cairo(&self, color: (u8, u8, u8, u8)) -> (f64, f64, f64, f64) {
        (
            color.0 as f64 / 255.0,
            color.1 as f64 / 255.0,
            color.2 as f64 / 255.0,
            color.3 as f64 / 255.0,
        )
    }

    /// Get the border color.
    pub fn get_border(&self) -> (f64, f64, f64, f64) {
        match self.custom.get_border() {
            Some(c) => c,
            None => {
                if self.is_dark {
                    self.to_cairo(self.dark.border)
                } else {
                    self.to_cairo(self.light.border)
                }
            }
        }
    }

    /// Get the background color.
    pub fn get_bg(&self) -> (f64, f64, f64, f64) {
        match self.custom.get_bg() {
            Some(c) => c,
            None => {
                if self.is_dark {
                    self.to_cairo(self.dark.bg)
                } else {
                    self.to_cairo(self.light.bg)
                }
            }
        }
    }

    /// Get the background color for the mapped cell.
    pub fn get_bg_map(&self) -> (f64, f64, f64, f64) {
        match self.custom.get_bg_map() {
            Some(c) => c,
            None => {
                if self.is_dark {
                    self.to_cairo(self.dark.bg_map)
                } else {
                    self.to_cairo(self.light.bg_map)
                }
            }
        }
    }

    /// Get the text color for the mapped cell.
    pub fn get_text(&self) -> (f64, f64, f64, f64) {
        match self.custom.get_text() {
            Some(c) => c,
            None => {
                if self.is_dark {
                    self.to_cairo(self.dark.text)
                } else {
                    self.to_cairo(self.light.text)
                }
            }
        }
    }

    /// Get the diamond color
    pub fn get_diamond(&self) -> (f64, f64, f64, f64) {
        match self.custom.get_border() {
            Some(c) => c,
            None => {
                if self.is_dark {
                    self.to_cairo(self.dark.diamond)
                } else {
                    self.to_cairo(self.light.diamond)
                }
            }
        }
    }

    /// Get the background color of cells with errors.
    pub fn get_text_wrong(&self) -> (f64, f64, f64, f64) {
        match self.custom.get_text_wrong() {
            Some(c) => c,
            None => {
                if self.is_dark {
                    self.to_cairo(self.dark.text_wrong)
                } else {
                    self.to_cairo(self.light.text_wrong)
                }
            }
        }
    }

    /// Get the selected cell background color.
    pub fn get_selection(&self) -> (f64, f64, f64, f64) {
        match self.custom.get_selection() {
            Some(c) => c,
            None => {
                if self.is_dark {
                    self.to_cairo(self.dark.selection)
                } else {
                    self.to_cairo(self.light.selection)
                }
            }
        }
    }

    /// Get the background color of successful cells.
    pub fn get_path(&self) -> (f64, f64, f64, f64) {
        match self.custom.get_path() {
            Some(c) => c,
            None => {
                if self.is_dark {
                    self.to_cairo(self.dark.path)
                } else {
                    self.to_cairo(self.light.path)
                }
            }
        }
    }

    /// Return the CSS for the puzzle background.
    pub fn get_bg_css(&self) -> &str {
        if self.is_dark {
            self.dark.bg_css
        } else {
            self.light.bg_css
        }
    }
}

/// Random puzzle parameters.
pub struct PuzzleSampleGame {
    /// Path as a list of cell IDs.
    pub path: Vec<u8>,

    /// List of diamonds.
    /// Each diamond is couple of cell IDs.
    pub diamonds: Vec<(u8, u8)>,

    /// List of mapped cells (cell IDs).
    pub map: Vec<u8>,
}

/// Parameters for the [`Puzzle`] object creation.
pub struct PuzzleParameters<'a> {
    /// Internal name of the puzzle.
    pub name: &'a str,

    /// Internationalized name of the puzzle, which is used for displaying in the interface.
    pub name_i18n: String,

    /// Difficulty level.
    pub difficulty: Difficulty,

    /// File name of the puzzle icon.
    pub icon: &'a str,

    /// File name of the Hexkudo logo.
    pub logo: &'a str,

    /// Puzzle colors for the light color theme.
    pub colors_light: PuzzleColor,

    /// Puzzle colors for the dark color theme.
    pub colors_dark: PuzzleColor,

    /// String representation of the puzzle.
    pub matrix: &'a str,

    /// Function to retrieve a static random puzzle in case generating the puzzle takes too long.
    pub get_sample_path_fn: fn() -> PuzzleSampleGame,
}

/// Puzzle parameters.
#[derive(Debug, Clone)]
pub struct Puzzle {
    /// Puzzle name.
    pub name: String,

    /// Translated puzzle name.
    pub name_i18n: String,

    /// Difficulty level.
    pub difficulty: Difficulty,

    /// Icon file in the Gio Resource.
    pub icon: String,

    /// PNG Image displayed in logo cells (Gio Resource).
    pub logo: String,

    /// Puzzle internal representation.
    pub matrix: puzzle_parse::PuzzleParse,

    /// Cell colors.
    pub colors: PuzzleColorTheme,

    /// Return a puzzle path from a sample path list.
    pub get_sample_path_fn: fn() -> PuzzleSampleGame,
}

impl Default for Puzzle {
    fn default() -> Self {
        Self {
            name: String::new(),
            name_i18n: String::new(),
            difficulty: Difficulty::Medium,
            icon: String::new(),
            logo: String::from("logo.png"),
            matrix: puzzle_parse::PuzzleParse::new(""),
            colors: PuzzleColorTheme {
                light: PuzzleColor {
                    border: (0, 0, 0, 0xFF),
                    bg: (0xFA, 0xFA, 0xFA, 0xFF),
                    bg_map: (0xE4, 0xE4, 0xE4, 0xFF),
                    text: (0, 0, 0, 0xFF),
                    diamond: (0, 0, 0, 0xFF),
                    text_wrong: (0x80, 0, 0, 0xFF),
                    selection: (0x91, 0xBC, 0xFF, 0xFF),
                    path: (0, 0, 0, 0x60),
                    bg_css: "",
                },
                dark: PuzzleColor {
                    border: (0xFF, 0xFF, 0xFF, 0xFF),
                    bg: (0x45, 0x45, 0x45, 0xFF),
                    bg_map: (0x33, 0x33, 0x33, 0xFF),
                    text: (0xFF, 0xFF, 0xFF, 0xFF),
                    diamond: (0xFF, 0xFF, 0xFF, 0xFF),
                    text_wrong: (0x80, 0, 0, 0xFF),
                    selection: (0, 0x42, 0x64, 0xFF),
                    path: (0xFF, 0xFF, 0xFF, 0x60),
                    bg_css: "",
                },
                custom: PuzzleCustomColor::new(),
                is_dark: false,
            },

            get_sample_path_fn: || -> PuzzleSampleGame {
                PuzzleSampleGame {
                    path: Vec::new(),
                    diamonds: Vec::new(),
                    map: Vec::new(),
                }
            },
        }
    }
}

impl Puzzle {
    /// Create a puzzle.
    fn new(parameters: PuzzleParameters) -> Self {
        Self {
            name: String::from(parameters.name),
            name_i18n: parameters.name_i18n,
            difficulty: parameters.difficulty,
            icon: String::from(parameters.icon),
            logo: String::from(parameters.logo),
            colors: PuzzleColorTheme {
                light: parameters.colors_light,
                dark: parameters.colors_dark,
                custom: PuzzleCustomColor::new(),
                is_dark: false,
            },
            matrix: puzzle_parse::PuzzleParse::new(parameters.matrix),
            get_sample_path_fn: parameters.get_sample_path_fn,
        }
    }

    /// Change the color theme.
    pub fn set_dark(&mut self, is_dark: bool) {
        self.colors.set_dark(is_dark);
    }
}

/// Return the puzzle list, indexed by name and difficulty.
pub fn puzzle_map() -> HashMap<(String, Difficulty), Puzzle> {
    let mut puzzles: HashMap<(String, Difficulty), Puzzle> = HashMap::new();

    // For developers: add your new puzzle to the list.
    let p: Puzzle = easy_classic_22::get();
    puzzles.insert((String::from(&p.name), p.difficulty), p);

    let p: Puzzle = easy_heart_24::get();
    puzzles.insert((String::from(&p.name), p.difficulty), p);

    let p: Puzzle = easy_square_22::get();
    puzzles.insert((String::from(&p.name), p.difficulty), p);

    let p: Puzzle = medium_classic_36::get();
    puzzles.insert((String::from(&p.name), p.difficulty), p);

    let p: Puzzle = medium_heart_45::get();
    puzzles.insert((String::from(&p.name), p.difficulty), p);

    let p: Puzzle = medium_square_38::get();
    puzzles.insert((String::from(&p.name), p.difficulty), p);

    let p: Puzzle = hard_classic_60::get();
    puzzles.insert((String::from(&p.name), p.difficulty), p);

    let p: Puzzle = hard_heart_58::get();
    puzzles.insert((String::from(&p.name), p.difficulty), p);

    let p: Puzzle = hard_square_60::get();
    puzzles.insert((String::from(&p.name), p.difficulty), p);

    puzzles
}
