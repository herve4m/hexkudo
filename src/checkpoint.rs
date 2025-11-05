/*
checkpoint.rs

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

//! Game checkpoint.
//!
//! Players can take checkpoints when resolving a puzzle so that they can revert back to a
//! well-known state if need be.
//! A checkpoint saves the currently completed cell values, the undo and redo lists, and the
//! currently selected cell.
//!
//! See [`crate::game`] where the checkpoints are organized as a stack: a player can create several
//!  checkpoints, and can revert back to the last checkpoint, which is then deleted.

use serde::{Deserialize, Serialize};

use crate::game::Game;
use crate::player_input::PlayerInput;

/// Checkpoint representation.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CheckPoint {
    /// List of the completed cell values and the undo and redo lists.
    pub player_input: PlayerInput,

    /// ID of the currently selected cell.
    pub selected_cell: Option<usize>,
}

impl CheckPoint {
    /// Create a [`CheckPoint`] object for the provided [`Game`] object.
    pub fn new(game: &Game) -> Self {
        Self {
            player_input: game.player_input.clone(),
            selected_cell: game.get_selected_cell(),
        }
    }
}
