/*
game.rs

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

//! Save and restore the game in progress when quitting or starting Hexkudo.
//!
//! When a game is in progress and the user quits Hexkudo, the game status is saved in the
//! `savegame.json` file.
//! When Hexkudo is restarted, the saved gave is loaded, and the user can continue the puzzle.
//!
//! The saved object is a serialization of the [`Game`] object in JSON format by using [`serde`].

use log::debug;
use std::error::Error;
use std::fmt;
use std::fs::{File, remove_file};
use std::io::{BufReader, BufWriter, ErrorKind, Write};
use std::path::PathBuf;

use serde::de::{self, Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};
use serde::ser::{Serialize, SerializeStruct, Serializer};

use crate::game::Game;
use crate::generator::puzzles;

/// Serialize and deserialize [`std::time::Instant`] objects with Serde.
pub mod instant {
    use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error};
    use std::time::{Duration, Instant};

    /// Serialize an [`std::time::Instant`] object.
    pub fn serialize<S>(instant: &Instant, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let duration: Duration = instant.elapsed();
        duration.serialize(serializer)
    }

    /// Deserialize an [`std::time::Instant`] object.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Instant, D::Error>
    where
        D: Deserializer<'de>,
    {
        let duration: Duration = Duration::deserialize(deserializer)?;
        let now: Instant = Instant::now();
        let instant: Instant = now
            .checked_sub(duration)
            .ok_or_else(|| Error::custom("Cannot compute the saved game duration"))?;
        Ok(instant)
    }
}

/// Serialize a [`puzzles::Puzzle`] object.
impl Serialize for puzzles::Puzzle {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // `2` is the number of fields to serialize
        let mut state = serializer.serialize_struct("Puzzle", 2)?;

        // Only serialize the puzzle name and difficulty. During deserialization, a complete
        // Puzzle object is retrieved from these two fields.
        state.serialize_field("name", &self.name)?;
        state.serialize_field("difficulty", &self.difficulty)?;
        state.end()
    }
}

/// Deserialize a [`puzzles::Puzzle`] object.
impl<'de> Deserialize<'de> for puzzles::Puzzle {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        enum Field {
            Name,
            Difficulty,
        }

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl Visitor<'_> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("`name` or `difficulty`")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            "name" => Ok(Field::Name),
                            "difficulty" => Ok(Field::Difficulty),
                            _ => Err(de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct PuzzleVisitor;

        impl<'de> Visitor<'de> for PuzzleVisitor {
            type Value = puzzles::Puzzle;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Puzzle")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<puzzles::Puzzle, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let name: String = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let difficulty: puzzles::Difficulty = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;

                // Retrieve the list (HashMap) of puzzles
                let puzzle_hash = puzzles::puzzle_map();
                // From this list, retrieve the Puzzle object that matches the serialized name and
                // difficulty, and then return that Puzzle.
                match puzzle_hash.get(&(name, difficulty)) {
                    Some(p) => Ok(p.clone()),
                    None => Err(de::Error::duplicate_field("puzzle")),
                }
            }

            fn visit_map<V>(self, mut map: V) -> Result<puzzles::Puzzle, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut name: Option<String> = None;
                let mut difficulty: Option<puzzles::Difficulty> = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Name => {
                            if name.is_some() {
                                return Err(de::Error::duplicate_field("name"));
                            }
                            name = Some(map.next_value()?);
                        }
                        Field::Difficulty => {
                            if difficulty.is_some() {
                                return Err(de::Error::duplicate_field("difficulty"));
                            }
                            difficulty = Some(map.next_value()?);
                        }
                    }
                }
                let name: String = name.ok_or_else(|| de::Error::missing_field("name"))?;
                let difficulty: puzzles::Difficulty =
                    difficulty.ok_or_else(|| de::Error::missing_field("difficulty"))?;
                // Retrieve the list (HashMap) of puzzles
                let puzzle_hash = puzzles::puzzle_map();
                // From this list, retrieve the Puzzle object that matches the serialized name and
                // difficulty, and then return that Puzzle.
                match puzzle_hash.get(&(name, difficulty)) {
                    Some(p) => Ok(p.clone()),
                    None => Err(de::Error::duplicate_field("puzzle")),
                }
            }
        }

        const FIELDS: &[&str] = &["secs", "nanos"];
        deserializer.deserialize_struct("Puzzle", FIELDS, PuzzleVisitor)
    }
}

/// Object to save and restore a puzzle in progress.
pub struct SaverGame {
    /// Absolute path to the save file.
    save_file: PathBuf,
}

impl SaverGame {
    /// Create a [`SaverGame`] object.
    ///
    /// The provided [`PathBuf`] is the path to the directory where the puzzle must be saved.
    pub fn new(mut data_dir: PathBuf) -> Self {
        data_dir.push("savegame.json");
        debug!("Save game file: {data_dir:?}");
        SaverGame {
            save_file: data_dir,
        }
    }

    /// Retrieve the [`Game`] object for the saved puzzle.
    ///
    /// Return the [`Game`] object or None if there is no saved puzzle.
    pub fn get_game(&self) -> Result<Option<Game>, Box<dyn Error>> {
        let file: File;
        match File::open(&self.save_file) {
            Ok(f) => file = f,
            Err(error) => match error.kind() {
                ErrorKind::NotFound => return Ok(None),
                _ => return Err(Box::new(error)),
            },
        }
        let reader: BufReader<File> = BufReader::new(file);
        let game: Game = serde_json::from_reader(reader)?;
        Ok(Some(game))
    }

    /// Save the provided [`Game`] object.
    pub fn save_game(&self, game: &Game) -> Result<(), Box<dyn Error>> {
        let file: File = File::create(&self.save_file)?;
        let mut writer: BufWriter<File> = BufWriter::new(file);

        serde_json::to_writer(&mut writer, game)?;
        writer.flush()?;
        Ok(())
    }

    /// Delete the saved game.
    pub fn delete_save(&self) {
        let _ = remove_file(&self.save_file);
    }
}
