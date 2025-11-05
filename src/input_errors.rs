/*
input_errors.rs

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

//! Manage the player's mistake counter.

use log::debug;
use std::collections::HashMap;
use std::time::Instant;

use serde::{Deserialize, Serialize};

/// The player gets three seconds to undo their mistake. This way the player can fix typo mistakes
/// or wrong cell selections without the mistake counter being incremented.
const TOLERATION_SEC: u64 = 3;

/// Manage the mistake counter.
#[derive(Serialize, Deserialize, Debug)]
pub struct InputErrors {
    // Number of errors.
    count: usize,

    // List of the cells in error, and the time the mistake was made. This enables decreasing the
    // error counter if the mistake is fixed in less that three seconds.
    #[serde(skip)]
    cell_set_time: HashMap<usize, Instant>,
}

impl InputErrors {
    /// Create an [`InputErrors`] object.
    pub fn new() -> Self {
        Self {
            count: 0,
            cell_set_time: HashMap::new(),
        }
    }

    /// Reset the object.
    pub fn clear(&mut self) {
        self.count = 0;
        self.cell_set_time.clear();
    }

    /// Return the number of mistakes.
    pub fn get_errors(&self) -> usize {
        self.count
    }

    /// Process the error status of the given cell, which the player just set.
    ///
    /// A cell in error increases the error counter.
    /// A cell with the correct value decreases the counter if the cell has been in error for less
    /// that three seconds.
    pub fn add_cell(&mut self, cell_id: usize, in_error: bool) {
        match self.cell_set_time.get(&cell_id) {
            // The cell has been in error previously
            Some(i) => {
                if in_error {
                    if i.elapsed().as_secs() > TOLERATION_SEC {
                        self.count += 1;
                        debug!("Error for cell {cell_id}: error count + 1 = {}", self.count);
                    }
                    self.cell_set_time.insert(cell_id, Instant::now());
                } else {
                    if i.elapsed().as_secs() <= TOLERATION_SEC && self.count > 0 {
                        self.count -= 1;
                        debug!(
                            "Error fixed for cell {cell_id} in less than {TOLERATION_SEC}s: error count - 1 = {}",
                            self.count
                        );
                    }
                    self.cell_set_time.remove(&cell_id);
                }
            }

            // The cell has not yet been in error
            None => {
                if in_error {
                    self.cell_set_time.insert(cell_id, Instant::now());
                    self.count += 1;
                    debug!("Error for cell {cell_id}: error count + 1 = {}", self.count);
                }
            }
        }
    }

    /// Process the error status of the given cell, which the player cleared.
    pub fn clear_cell(&mut self, cell_id: usize) {
        if let Some(i) = self.cell_set_time.get(&cell_id) {
            if i.elapsed().as_secs() <= TOLERATION_SEC && self.count > 0 {
                self.count -= 1;
                debug!(
                    "Removed cell {cell_id} in less than {TOLERATION_SEC}s: error count - 1 = {}",
                    self.count
                );
            }
            self.cell_set_time.remove(&cell_id);
        }
    }
}
