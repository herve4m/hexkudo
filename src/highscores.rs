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

//! Manage high scores for the puzzles.
//!
//! The main object, [`HighScores`], maintains a list of high scores for each puzzle.
//! This object is saved when the user completes a puzzle and makes it to the scoreboard, and
//! is restored when Hexkudo starts.
//! See the [`crate::saver::highscores`] module that saves and restores the [`HighScores`] object.

use std::collections::HashMap;
use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};

use crate::generator::puzzles;

/// Number of entries per scoreboard (number of top scores to keep).
const BOARD_SIZE: usize = 10;

/// Object that represent a score.
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Score {
    /// How long did it take for solving the puzzle.
    pub time: Duration,

    /// Number of mistakes while resolving the puzzle.
    pub errors: usize,

    /// Completion timestamp, which is used to display the date and time in the scoreboard.
    pub when: SystemTime,
}

/// Sorted list of the top scores for a puzzle.
#[derive(Serialize, Deserialize, Debug, Clone)]
struct PuzzleHighScoreBoard {
    /// Sorted list of the top scores.
    /// The number of scores in this list is controlled by the [`BOARD_SIZE`] constant.
    top: Vec<Score>,
}

impl PuzzleHighScoreBoard {
    /// Create a [`PuzzleHighScoreBoard`] object.
    fn new() -> Self {
        Self {
            top: Vec::with_capacity(BOARD_SIZE),
        }
    }

    /// Add a score to the scoreboard and return the position in the board, or None if the
    /// score does not make it to the board.
    ///
    /// The returned position starts at 1 (top score).
    fn add_score(&mut self, time: Duration, errors: usize) -> Option<usize> {
        let mut new_score_position: Option<usize> = None;
        let mut tmp_top: Vec<Score> = Vec::with_capacity(BOARD_SIZE);
        let mut i: usize = 0;

        for score in &self.top {
            // Insert the new score to the temporary board
            if time < score.time && new_score_position.is_none() {
                new_score_position = Some(i + 1);
                tmp_top.push(Score {
                    time,
                    errors,
                    when: SystemTime::now(),
                });
                i += 1;
            }
            // Do not add more scores than the board size
            if i >= BOARD_SIZE {
                break;
            }
            tmp_top.push(*score);
            i += 1;
        }
        // If the board is not full and the new score has not been added yet, then add the new
        // score at the end of the board
        if i < BOARD_SIZE && new_score_position.is_none() {
            new_score_position = Some(i + 1);
            tmp_top.push(Score {
                time,
                errors,
                when: SystemTime::now(),
            });
        }
        self.top = tmp_top;
        new_score_position
    }
}

/// List of the scoreboards for the puzzles.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HighScores {
    /// Map fo the [`PuzzleHighScoreBoard`] scoreboards indexed by the puzzle.
    ///
    /// The puzzle index is a string in the format "<puzzle_name>@@<difficulty>".
    board: HashMap<String, PuzzleHighScoreBoard>,
}

impl Default for HighScores {
    fn default() -> Self {
        Self::new()
    }
}

impl HighScores {
    /// Create a [`HighScores`] object.
    pub fn new() -> Self {
        Self {
            board: HashMap::new(),
        }
    }

    /// Return the string that is used as an index for the list of scoreboards.
    fn build_key(&self, puzzle_name: &String, difficulty: puzzles::Difficulty) -> String {
        format!("{puzzle_name}@@{difficulty}")
    }

    /// Add the a score to the scoreboard of the provided puzzle and return the position in the
    /// scoreboard, or None if the score does not make it to the board.
    ///
    /// The returned position starts at 1 (top score).
    pub fn add_score(
        &mut self,
        puzzle_name: &String,
        difficulty: puzzles::Difficulty,
        time: Duration,
        errors: usize,
    ) -> Option<usize> {
        let key: String = self.build_key(puzzle_name, difficulty);
        let scoreboard: &mut PuzzleHighScoreBoard =
            self.board.entry(key).or_insert(PuzzleHighScoreBoard::new());

        scoreboard.add_score(time, errors)
    }

    /// Return the list of [`Score`] for the given puzzle.
    ///
    /// Return None when the scoreboard is empty.
    pub fn get_score(
        &self,
        puzzle_name: &String,
        difficulty: puzzles::Difficulty,
    ) -> Option<&Vec<Score>> {
        let key: String = self.build_key(puzzle_name, difficulty);

        match self.board.get(&key) {
            Some(b) => Some(&b.top),
            None => None,
        }
    }

    /// Return whether the list of scoreboard is empty (no scoreboard for any puzzle)
    pub fn is_empty(&self) -> bool {
        self.board.len() == 0
    }
}
