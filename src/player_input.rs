/*
player_input.rs

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

//! Manage the player's cell input.
//!
//! The module manges the cell values that the player entered, as well as the undo and redo lists.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Undo and redo operations.
#[derive(Serialize, Deserialize, Debug, Clone)]
enum Operation {
    Add,
    Remove,
}

/// Cell parameters for an undo and redo operation.
/// The object stores the operation that was performed by the player.
#[derive(Serialize, Deserialize, Debug, Clone)]
struct DoOperation {
    /// Operation: [`Operation::Add`] or [`Operation::Remove`].
    operation: Operation,

    /// Cell ID.
    cell_id: usize,

    /// Cell value.
    cell_value: usize,
}

/// Manage the puzzle cells that the player completed.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlayerInput {
    /// List of the cell IDs and their associated values.
    id_to_value: HashMap<usize, usize>,

    /// List of the cell values and the associated cell IDs.
    /// Each value might have several cell IDs, when the player mistakenly entered the same value
    /// in two or more cells.
    value_to_ids: HashMap<usize, Vec<usize>>,

    /// List of undo operations.
    undo_op: Vec<DoOperation>,

    /// List of redo operations.
    redo_op: Vec<DoOperation>,
}

impl Default for PlayerInput {
    fn default() -> Self {
        Self::new()
    }
}

impl PlayerInput {
    /// Create a [`PlayerInput`] object.
    pub fn new() -> Self {
        Self {
            id_to_value: HashMap::new(),
            value_to_ids: HashMap::new(),
            undo_op: Vec::new(),
            redo_op: Vec::new(),
        }
    }

    /// Reset the object.
    pub fn clear(&mut self) {
        self.id_to_value.clear();
        self.value_to_ids.clear();
        self.undo_op.clear();
        self.redo_op.clear();
    }

    /// Return the cell values in an [`HashMap`] indexed by the cell IDs.
    pub fn get_values(&self) -> &HashMap<usize, usize> {
        &self.id_to_value
    }

    /// Return a cell's value or None is the player has not filled that cell yet.
    pub fn get_value_from_id(&self, cell_id: usize) -> Option<usize> {
        self.id_to_value.get(&cell_id).copied()
    }

    /// Return the cell ID that has the given value, or None if the player has not entered that
    /// value yet, of if the player used the same value in several cells.
    pub fn get_id_from_value(&self, cell_value: usize) -> Option<usize> {
        match self.value_to_ids.get(&cell_value) {
            Some(values) => {
                if values.len() == 1 {
                    Some(values[0])
                } else {
                    None
                }
            }
            None => None,
        }
    }

    /// Whether a cell has the provided value.
    pub fn contains_value(&self, cell_value: usize) -> bool {
        match self.value_to_ids.get(&cell_value) {
            Some(v) => !v.is_empty(),
            None => false,
        }
    }

    /// Return the number of cells that the player already completed (maybe with incorrect values)
    pub fn len(&self) -> usize {
        self.id_to_value.len()
    }

    /// Add a value to a cell, but do not store the operation in the undo list.
    pub fn add_no_undo(&mut self, cell_id: usize, cell_value: usize) {
        self.id_to_value.insert(cell_id, cell_value);
        match self.value_to_ids.get_mut(&cell_value) {
            Some(v) => {
                if !v.contains(&cell_id) {
                    v.push(cell_id);
                }
            }
            None => {
                self.value_to_ids.insert(cell_value, vec![cell_id]);
            }
        }
    }

    /// Add a value to a cell and add the operation to the undo list.
    pub fn add(&mut self, cell_id: usize, cell_value: usize) {
        // First, remove the previous value
        self.remove(cell_id);
        self.add_no_undo(cell_id, cell_value);
        self.undo_op.push(DoOperation {
            operation: Operation::Add,
            cell_id,
            cell_value,
        });
        self.redo_op.clear();
    }

    /// Remove the value from the given cell and return the removed value or None if the cell
    /// had no value.
    /// Do not update the undo list.
    fn remove_no_undo(&mut self, cell_id: usize) -> Option<usize> {
        match self.id_to_value.remove(&cell_id) {
            Some(cell_value) => {
                // Remove the cell ID from value-to-cell vector.
                if let Some(v) = self.value_to_ids.get_mut(&cell_value) {
                    v.retain(|id| *id != cell_id);
                }
                Some(cell_value)
            }
            None => None,
        }
    }

    /// Remove the value from the given cell.
    pub fn remove(&mut self, cell_id: usize) {
        if let Some(cell_value) = self.remove_no_undo(cell_id) {
            self.undo_op.push(DoOperation {
                operation: Operation::Remove,
                cell_id,
                cell_value,
            });
            self.redo_op.clear();
        }
    }

    /// Whether the player entered the given value in multiple cells, which is a mistake.
    pub fn is_value_duplicated(&self, cell_value: usize) -> bool {
        match self.value_to_ids.get(&cell_value) {
            Some(v) => v.len() > 1,
            None => false,
        }
    }

    /// Undo the last operation.
    pub fn undo(&mut self) {
        if let Some(op) = self.undo_op.pop() {
            match op.operation {
                Operation::Add => {
                    self.remove_no_undo(op.cell_id);
                }
                Operation::Remove => {
                    self.add_no_undo(op.cell_id, op.cell_value);
                }
            }
            self.redo_op.push(op);
        }
    }

    /// Redo the last undo operation.
    pub fn redo(&mut self) {
        if let Some(op) = self.redo_op.pop() {
            match op.operation {
                Operation::Add => {
                    self.add_no_undo(op.cell_id, op.cell_value);
                }
                Operation::Remove => {
                    self.remove_no_undo(op.cell_id);
                }
            }
            self.undo_op.push(op);
        }
    }

    /// Return the length of the undo list.
    pub fn undo_len(&self) -> usize {
        self.undo_op.len()
    }

    /// Return the length of the redo list.
    pub fn redo_len(&self) -> usize {
        self.redo_op.len()
    }
}
