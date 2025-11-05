/*
highscores.rs

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

//! Save and restore the high scores for the puzzles.
//!
//! The saved object is a serialization of the [`HighScores`] object in JSON format by
//! using [`serde`].

use log::debug;
use std::error::Error;
use std::fs::{File, remove_file};
use std::io::{BufReader, BufWriter, ErrorKind, Write};
use std::path::PathBuf;

use crate::highscores::HighScores;

/// Object to save and restore a high scores.
pub struct SaverHighScores {
    /// Absolute path to the save file.
    save_file: PathBuf,
}

impl SaverHighScores {
    /// Create a [`SaverHighScores`] object.
    ///
    /// The provided [`PathBuf`] is the path to the directory where the high scores must be saved.
    pub fn new(mut data_dir: PathBuf) -> Self {
        data_dir.push("highscores.json");
        debug!("High scores file: {data_dir:?}");
        Self {
            save_file: data_dir,
        }
    }

    /// Retrieve the [`HighScores`] object for the high scores file.
    ///
    /// Return the [`HighScores`] object or None if the high scores file does not exist.
    pub fn get_highscores(&self) -> Result<Option<HighScores>, Box<dyn Error>> {
        let file: File;
        match File::open(&self.save_file) {
            Ok(f) => file = f,
            Err(error) => match error.kind() {
                ErrorKind::NotFound => return Ok(None),
                _ => return Err(Box::new(error)),
            },
        }
        let reader: BufReader<File> = BufReader::new(file);
        let highscores: HighScores = serde_json::from_reader(reader)?;
        Ok(Some(highscores))
    }

    /// Save the provided [`HighScores`] object.
    pub fn save_highscores(&self, highscores: &HighScores) -> Result<(), Box<dyn Error>> {
        let file: File = File::create(&self.save_file)?;
        let mut writer: BufWriter<File> = BufWriter::new(file);

        serde_json::to_writer(&mut writer, highscores)?;
        writer.flush()?;
        Ok(())
    }

    /// Delete the high scores file.
    pub fn delete_save(&self) {
        let _ = remove_file(&self.save_file);
    }
}
