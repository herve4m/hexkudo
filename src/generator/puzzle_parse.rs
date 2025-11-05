/*
puzzle_parse.rs

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

//! Parse an "ASCII art" representation of an Hexkudo puzzle.
//!
//! The [`PuzzleParse`] object groups the vertexes and the edges after parsing.

use log::{Level, log_enabled};
use serde::{Deserialize, Serialize};

use super::edges;
use super::vertexes;

/// Puzzle parsing object.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PuzzleParse {
    /// List of the edges in the puzzle after the puzzle has been parsed.
    pub edges: edges::Edges,

    /// Vertex matrix.
    pub vertexes: vertexes::Vertexes,
}

/// Puzzle parsing object.
impl PuzzleParse {
    /// Create a [`PuzzleParse`] object.
    pub fn new(puzzle: &str) -> Self {
        Self {
            edges: edges::Edges::new(),
            vertexes: vertexes::Vertexes::new(puzzle),
        }
    }

    /// Parse the source puzzle and build its internal representation.
    ///
    /// # Errors
    ///
    /// The method returns an error when the source puzzle as errors, such as an isolated vertex
    /// (with no edges), or several vertexes with only one edge (only one such vertex is allowed;
    /// it becomes the starting vertex).
    pub fn build_edges(&mut self) -> Result<(), String> {
        // Parse the source puzzle in its intermediate representation
        if self.vertexes.num_vertexes == 0 {
            self.vertexes.build();
        }

        self.edges.clear();
        self.vertexes.required_starting_vertex = None;

        for y in 0..self.vertexes.height {
            for x in 0..self.vertexes.width {
                let v1: usize = match self.vertexes.get_cell(x, y) {
                    vertexes::CellType::Vertex(num) => num,
                    _ => {
                        continue;
                    }
                };

                let mut e: Vec<usize> = Vec::new();

                // Row above the current vertex
                if y > 0 {
                    // Top left
                    if x > 0 {
                        self.push_edge(&mut e, self.vertexes.get_cell(x - 1, y - 1));
                    }
                    // Top right
                    self.push_edge(&mut e, self.vertexes.get_cell(x + 1, y - 1));
                }

                // Current row
                if x >= 2 {
                    self.push_edge(&mut e, self.vertexes.get_cell(x - 2, y));
                }
                self.push_edge(&mut e, self.vertexes.get_cell(x + 2, y));

                // Row below the current vertex
                // Bottom left
                if x > 0 {
                    self.push_edge(&mut e, self.vertexes.get_cell(x - 1, y + 1));
                }
                // Bottom right
                self.push_edge(&mut e, self.vertexes.get_cell(x + 1, y + 1));

                let num_edges: usize = e.len();
                if num_edges == 0 {
                    return Err(format!("Vertex {v1} does not have any edges"));
                }
                if num_edges == 1 {
                    if self.vertexes.required_starting_vertex.is_some() {
                        return Err(format!(
                            "Vertexes {} and {} have only one edge (only one such vertex is allowed)",
                            self.vertexes.required_starting_vertex.unwrap(),
                            v1
                        ));
                    }
                    self.vertexes.required_starting_vertex = Some(v1);
                }
                self.edges
                    .push_from_array(v1, &e, edges::EdgeStatus::Undecided);
            }
        }

        if log_enabled!(Level::Debug) {
            self.edges.debug();
        }
        Ok(())
    }

    /// Add a vertex to an array of edges.
    fn push_edge(&self, edge: &mut Vec<usize>, cell: vertexes::CellType) {
        if let vertexes::CellType::Vertex(v2) = cell {
            edge.push(v2);
        }
    }
}
