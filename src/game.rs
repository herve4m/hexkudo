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

//! Manage the status of a game in progress.

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

use crate::checkpoint::CheckPoint;
use crate::generator::diamond_and_map::DiamondAndMap;
use crate::generator::path::Path;
use crate::generator::puzzles::Puzzle;
use crate::generator::vertexes;
use crate::input_errors::InputErrors;
use crate::player_input::PlayerInput;
use crate::saver::game::instant;

/// Status of a cell that the player completed.
pub struct CellStatus {
    /// Cell identifier.
    pub cell_id: usize,

    /// Value set by the player.
    pub cell_value: usize,

    /// Whether several cells have the same value.
    pub duplicated: bool,

    /// Whether the player entered the wrong value.
    pub error: bool,
}

/// Manage the status of the game in progress.
#[derive(Serialize, Deserialize, Debug)]
pub struct Game {
    /// List of the cells that the player completed.
    pub player_input: PlayerInput,

    /// Identifier of the selected cell.
    selected_cell: Option<usize>,

    /// Whether the player started entering a value with the keyboard in the selected cell.
    /// This is used to determine if the player input is the next digit of the value.
    selected_cell_value_updated: bool,

    /// Puzzle details.
    pub puzzle: Puzzle,

    /// Current puzzle path.
    pub path: Path,

    /// List of mapped cells (hints).
    pub map: Vec<usize>,

    /// List of diamonds.
    pub diamonds: Vec<(usize, usize)>,

    /// Whether the player asked for a cell value or for solving the puzzle (those are options in
    /// the menu). In this case the user time is not added to the score board.
    pub user_has_cheated: bool,

    /// Whether the player paused the game. In that case, the game board id hidden.
    pub paused: bool,

    /// Whether the game has started.
    pub started: bool,

    /// Whether the puzzle is solved.
    pub solved: bool,

    /// Time when the game started. Used to compute game duration.
    #[serde(with = "instant")]
    start_time: Instant,

    /// The elapsed time when the player paused the game.
    pause_duration: Option<Duration>,

    /// List of checkpoints set by the player.
    checkpoints: Vec<CheckPoint>,

    /// Manage input errors and the mistake counter.
    input_errors: InputErrors,
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

impl Game {
    /// Create a [`Game`] object.
    pub fn new() -> Self {
        Self {
            player_input: PlayerInput::new(),
            selected_cell: None,
            selected_cell_value_updated: false,
            puzzle: Puzzle::default(),
            path: Path::default(),
            map: Vec::new(),
            diamonds: Vec::new(),
            user_has_cheated: false,
            paused: false,
            started: false,
            solved: false,
            start_time: Instant::now(),
            pause_duration: None,
            checkpoints: Vec::new(),
            input_errors: InputErrors::new(),
        }
    }

    /// Clear all resources.
    pub fn clear(&mut self) {
        self.player_input.clear();
        self.selected_cell = None;
        self.selected_cell_value_updated = false;
        self.puzzle = Puzzle::default();
        self.path.clear();
        self.map.clear();
        self.diamonds.clear();
        self.user_has_cheated = false;
        self.paused = false;
        self.started = false;
        self.solved = false;
        self.pause_duration = None;
        self.checkpoints.clear();
        self.input_errors.clear();
    }

    /// Restart the game status (player inputs), but keep the puzzle data (structure, errors, timer)
    pub fn reset(&mut self) {
        self.player_input.clear();
        self.checkpoints.clear();
        self.init_path();
        self.paused = false;
        self.started = true;
        if self.solved {
            self.start_time = Instant::now();
            self.solved = false;
        }
    }

    /// Change the currently selected cell.
    pub fn set_selected_cell(&mut self, cell_id: Option<usize>) {
        if cell_id == self.selected_cell {
            return;
        }
        self.selected_cell = cell_id;
        self.selected_cell_value_updated = false;
    }

    /// Get the cell ID of the selected cell.
    pub fn get_selected_cell(&self) -> Option<usize> {
        self.selected_cell
    }

    /// Get the cell ID and the value of the selected cell.
    pub fn get_selected_cell_value(&self) -> Option<(usize, usize)> {
        match self.selected_cell {
            Some(cid) => self.path.vertex_index(cid).map(|v| (cid, v + 1)),
            None => None,
        }
    }

    /// Whether the value of the selected cell has been updated since the player moved
    /// the selection.
    pub fn is_selected_cell_value_updated(&self) -> bool {
        self.selected_cell_value_updated
    }

    /// Set the updated status of the selected cell.
    pub fn set_selected_cell_value_updated(&mut self, updated: bool) {
        if self.selected_cell.is_some() {
            self.selected_cell_value_updated = updated;
        }
    }

    /// Provide the [`Puzzle`] object being played.
    pub fn set_puzzle(&mut self, puzzle: &Puzzle) {
        self.clear();
        self.puzzle = puzzle.clone();
    }

    /// Provide the details of the puzzle (hints and diamonds).
    pub fn set_path(&mut self, path: &Path, d_and_m: &DiamondAndMap) {
        self.path = path.clone();
        (self.diamonds, self.map) = d_and_m.get_diamond_and_map();
        self.init_path();
        self.started = true;
        self.start_time = Instant::now();
    }

    /// Initialize the game: declare the mapped (hint) cells and choose the first selected cell.
    fn init_path(&mut self) {
        // Add the map (hint) cells to the user input to indicate that they are solved.
        for cell_id in &self.map {
            if let Some(v) = self.path.vertex_index(*cell_id) {
                self.player_input.add_no_undo(*cell_id, v + 1);
            }
        }

        // For the initial selected cell, choose a cell close to the starting cell
        for cell_id in self.path.get() {
            let adjacent: vertexes::Adjacent = self.puzzle.matrix.vertexes.get_adjacent(*cell_id);

            if let Some(cell_type) = adjacent.w
                && let vertexes::CellType::Vertex(c) = cell_type
                && !self.map.contains(&c)
            {
                self.selected_cell = Some(c);
                break;
            }

            if let Some(cell_type) = adjacent.nw
                && let vertexes::CellType::Vertex(c) = cell_type
                && !self.map.contains(&c)
            {
                self.selected_cell = Some(c);
                break;
            }

            if let Some(cell_type) = adjacent.ne
                && let vertexes::CellType::Vertex(c) = cell_type
                && !self.map.contains(&c)
            {
                self.selected_cell = Some(c);
                break;
            }

            if let Some(cell_type) = adjacent.e
                && let vertexes::CellType::Vertex(c) = cell_type
                && !self.map.contains(&c)
            {
                self.selected_cell = Some(c);
                break;
            }

            if let Some(cell_type) = adjacent.se
                && let vertexes::CellType::Vertex(c) = cell_type
                && !self.map.contains(&c)
            {
                self.selected_cell = Some(c);
                break;
            }

            if let Some(cell_type) = adjacent.sw
                && let vertexes::CellType::Vertex(c) = cell_type
                && !self.map.contains(&c)
            {
                self.selected_cell = Some(c);
                break;
            }
        }
        self.selected_cell_value_updated = false;
    }

    /// Whether the puzzle is successfully solved.
    pub fn is_solved(&mut self) -> bool {
        // Return if not all cells have values
        if self.player_input.len() < self.puzzle.matrix.vertexes.num_vertexes - self.map.len() {
            return false;
        }

        // A previous call already established that the puzzle was solved
        if self.solved {
            return true;
        }

        for (i, vertex_id) in self.path.get().iter().enumerate() {
            // The cell is in the map list (hints). That's fine so continue verifying the other
            // cells.
            if self.map.contains(vertex_id) {
                continue;
            }
            match self.player_input.get_value_from_id(*vertex_id) {
                Some(v) => {
                    // The user entered a wrong value for the cell
                    if v != i + 1 {
                        return false;
                    }
                }
                None => {
                    // The user did not entered a value for the cell (that should not be possible
                    // at that stage)
                    return false;
                }
            }
        }
        self.solved = true;
        true
    }

    /// Return the number of errors so far.
    pub fn get_errors(&self) -> usize {
        self.input_errors.get_errors()
    }

    /// Return the number of checkpoints that the player created.
    pub fn checkpoints_len(&self) -> usize {
        self.checkpoints.len()
    }

    /// Set a checkpoint.
    pub fn set_checkpoint(&mut self) {
        self.checkpoints.push(CheckPoint::new(self));
    }

    /// Revert back to the last checkpoint.
    pub fn undo_checkpoint(&mut self) {
        // The checkpoint is removed
        if let Some(c) = self.checkpoints.pop() {
            self.player_input = c.player_input;
            self.selected_cell = c.selected_cell;
        }
    }

    /// Whether the given value is the correct value for the given cell ID.
    fn is_cell_error(&self, cell_id: usize, cell_value: usize) -> bool {
        match self.path.get_vertex_from_value(cell_value) {
            Some(cid) => cid != cell_id,
            None => true,
        }
    }

    /// Get the list of all the cells that the player completed as well as the mapped cells.
    /// For each cell, the [`CellStatus`] object indicate whether the value is wrong and/or
    /// duplicated.
    pub fn get_cells(&self) -> Vec<CellStatus> {
        let mut ret: Vec<CellStatus> = Vec::with_capacity(self.path.len());

        for (cell_id, cell_value) in self.player_input.get_values() {
            ret.push(CellStatus {
                cell_id: *cell_id,
                cell_value: *cell_value,
                duplicated: self.player_input.is_value_duplicated(*cell_value),
                error: self.is_cell_error(*cell_id, *cell_value),
            });
        }
        ret
    }

    /// Add the value that the player provided to the given cell.
    pub fn add_value_to_cell(&mut self, cell_id: usize, cell_value: usize) {
        self.player_input.add(cell_id, cell_value);
        // Verify whether this is the correct value. If not, then the error counter is incremented.
        self.input_errors
            .add_cell(cell_id, self.is_cell_error(cell_id, cell_value));
    }

    /// Remove the value of the given cell.
    pub fn remove_value_from_cell(&mut self, cell_id: usize) {
        self.player_input.remove(cell_id);
        self.input_errors.clear_cell(cell_id);
    }

    /// Pause the game.
    pub fn pause(&mut self) {
        // Store the played time so far, so that the pause time can be deduced when the
        // player resumes the game.
        self.pause_duration = Some(self.start_time.elapsed());
        self.paused = true;
    }

    /// Resume the game.
    pub fn resume(&mut self) {
        // Refresh the game elapsed time by removing the pause time.
        if let Some(d) = self.pause_duration {
            self.start_time += self.start_time.elapsed() - d;
            self.pause_duration = None;
        }
        self.paused = false;
    }

    /// Return the game duration.
    pub fn get_duration(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Return the game duration in hours, minutes, and seconds
    pub fn get_duration_hms(&self) -> (u64, u64, u64) {
        let duration: u64 = self.start_time.elapsed().as_secs();
        (
            duration / 3600,
            (duration % 3600) / 60,
            (duration % 3600) % 60,
        )
    }
}
