/*
vertexes.rs

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

//! Vertexes for cells in the Hexkudo graph.

use log::{Level, debug, log_enabled};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Representation of an Hexkudo cell from the "ASCII art" representation.
///
/// - A `Background` cell is a cell outside the puzzle.
/// - A `Logo` cell is a cell which is not part of the puzzle, but must be represented as the
///   Hexkudo logo.
/// - A `Vertex` represents a vertex in the puzzle, with its ID.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Default, PartialEq)]
pub enum CellType {
    #[default]
    Background,
    Logo,
    Vertex(usize),
}

/// Cells adjacent to a given cell.
#[derive(Debug, Copy, Clone, Default)]
pub struct Adjacent {
    pub w: Option<CellType>,
    pub nw: Option<CellType>,
    pub ne: Option<CellType>,
    pub e: Option<CellType>,
    pub se: Option<CellType>,
    pub sw: Option<CellType>,
}

impl Adjacent {
    // Whether the two given cells are on opposite sides.
    pub fn opposite(&self, vertex1: usize, vertex2: usize) -> bool {
        // West and east
        if self.w.is_some()
            && self.e.is_some()
            && let CellType::Vertex(v1) = self.w.unwrap()
            && (v1 == vertex1 || v1 == vertex2)
            && let CellType::Vertex(v2) = self.e.unwrap()
            && (v2 == vertex1 || v2 == vertex2)
        {
            return true;
        }
        // North-west and south-east
        if self.nw.is_some()
            && self.se.is_some()
            && let CellType::Vertex(v1) = self.nw.unwrap()
            && (v1 == vertex1 || v1 == vertex2)
            && let CellType::Vertex(v2) = self.se.unwrap()
            && (v2 == vertex1 || v2 == vertex2)
        {
            return true;
        }
        // North-east and south-west
        if self.ne.is_some()
            && self.sw.is_some()
            && let CellType::Vertex(v1) = self.ne.unwrap()
            && (v1 == vertex1 || v1 == vertex2)
            && let CellType::Vertex(v2) = self.sw.unwrap()
            && (v2 == vertex1 || v2 == vertex2)
        {
            return true;
        }
        false
    }
}

/// Represent the vertexes of the Hexkudo graph.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Vertexes {
    /// Number of vertexes discovered after the "ASCII art" parsing.
    pub num_vertexes: usize,

    /// Number of characters in the widest part of the puzzle.
    pub width: usize,

    /// Height of the puzzle in characters.
    pub height: usize,

    /// ID of the vertex that must be used as a starting point in the puzzle.
    /// During parsing, if the process detects a vertex with only one possible edge, then this
    /// vertex is designed as the starting vertex.
    /// Otherwise, [`Vertexes::required_starting_vertex`] is set to [`None`], and the path finder
    /// will randomly select a starting vertex.
    /// Only one such vertex can be present in the puzzle, otherwise an error is raised.
    pub required_starting_vertex: Option<usize>,

    /// Source "ASCII art" puzzle such as:
    ///
    /// ```
    ///    O O
    ///   O O O
    ///  O O O O
    /// O O X O O
    ///  O O O O
    ///   O O O
    ///    O O
    /// ```
    ///
    /// - `o` or `O` represents a puzzle cell.
    /// - `x` or `X` represents the Hexkudo logo.
    /// - All other characters are ignored (background)
    puzzle_source: String,

    /// Puzzle's vertexes.
    vertex_array: Vec<Vec<CellType>>,

    /// Vertex coordinates.
    vertex_coordinates: HashMap<usize, (usize, usize)>,

    /// Logo coordinates.
    logo_coordinates: Vec<(usize, usize)>,
}

impl Vertexes {
    /// Create a [`Vertexes`] object.
    pub fn new(puzzle: &str) -> Self {
        Self {
            num_vertexes: 0,
            width: 0,
            height: 0,
            required_starting_vertex: None,
            puzzle_source: puzzle.to_lowercase(),
            vertex_array: Vec::new(),
            vertex_coordinates: HashMap::new(),
            logo_coordinates: Vec::new(),
        }
    }

    /// Convert the source "ASCII art" puzzle into a vertex matrix.
    pub fn build(&mut self) {
        let mut max_width: usize = 0;
        let mut cell_number: usize = 0;
        let mut x: usize;
        let mut y: usize = 0;

        self.vertex_array.clear();

        for row in self.puzzle_source.lines() {
            let r: &str = row.trim_end();
            let row_length: usize = r.len();
            if row_length == 0 {
                continue;
            }
            if row_length > max_width {
                max_width = row_length;
            }
            let mut cols: Vec<CellType> = Vec::new();
            x = 0;
            for c in r.chars() {
                match c {
                    'o' => {
                        self.vertex_coordinates.insert(cell_number, (x, y));
                        cols.push(CellType::Vertex(cell_number));
                        cell_number += 1;
                    }
                    'x' => {
                        self.logo_coordinates.push((x, y));
                        cols.push(CellType::Logo);
                    }
                    _ => cols.push(CellType::Background),
                }
                x += 1;
            }
            y += 1;
            self.vertex_array.push(cols);
        }

        // Fill the end of the rows with background cells to get a square vertex matrix
        for v in &mut self.vertex_array {
            v.append(&mut vec![CellType::Background; max_width - v.len()]);
        }

        self.num_vertexes = cell_number;
        self.width = max_width;
        self.height = self.vertex_array.len();

        if log_enabled!(Level::Debug) {
            debug!("Number of vertexes: {}", self.num_vertexes);
            debug!("             width: {}", self.width);
            debug!("            height: {}", self.height);

            let mut s: String = String::new();
            for y in 0..self.height {
                s.clear();
                for x in 0..self.width {
                    match self.get_cell(x, y) {
                        CellType::Logo => s.push_str(" X "),
                        CellType::Background => s.push_str(" . "),
                        CellType::Vertex(cell) => s.push_str(&format!("{cell:^3}")),
                    }
                }
                debug!("{s}");
            }
        }
    }

    /// Return the coordinates of the given cell.
    pub fn get_coordinates(&self, cell_id: usize) -> Option<(usize, usize)> {
        self.vertex_coordinates.get(&cell_id).map(|c| (c.0, c.1))
    }

    /// Return the coordinates of the logo.
    pub fn get_logo_coordinates(&self) -> &[(usize, usize)] {
        &self.logo_coordinates[..]
    }

    /// Get the cell at the given coordinates.
    pub fn get_cell(&self, x: usize, y: usize) -> CellType {
        if x >= self.width || y >= self.height {
            CellType::Background
        } else {
            self.vertex_array[y][x]
        }
    }

    /// Return the cells adjacent to the provided cell.
    pub fn get_adjacent(&self, cell_id: usize) -> Adjacent {
        match self.get_coordinates(cell_id) {
            None => Adjacent {
                w: None,
                nw: None,
                ne: None,
                e: None,
                se: None,
                sw: None,
            },
            Some((x, y)) => {
                // West
                let w: Option<CellType> = if x >= 2 {
                    Some(self.get_cell(x - 2, y))
                } else {
                    None
                };

                // North-west
                let nw: Option<CellType> = if x >= 1 && y >= 1 {
                    Some(self.get_cell(x - 1, y - 1))
                } else {
                    None
                };

                // North-east
                let ne: Option<CellType> = if x <= self.width - 2 && y >= 1 {
                    Some(self.get_cell(x + 1, y - 1))
                } else {
                    None
                };

                // East
                let e: Option<CellType> = if x <= self.width - 3 {
                    Some(self.get_cell(x + 2, y))
                } else {
                    None
                };

                // South-east
                let se: Option<CellType> = if x <= self.width - 2 && y <= self.height - 2 {
                    Some(self.get_cell(x + 1, y + 1))
                } else {
                    None
                };

                // South-west
                let sw: Option<CellType> = if x >= 1 && y <= self.height - 2 {
                    Some(self.get_cell(x - 1, y + 1))
                } else {
                    None
                };

                Adjacent {
                    w,
                    nw,
                    ne,
                    e,
                    se,
                    sw,
                }
            }
        }
    }

    /// Whether two cells are adjacent.
    pub fn is_adjacent(&self, cell_id_1: usize, cell_id_2: usize) -> bool {
        let adjacent: Adjacent = self.get_adjacent(cell_id_1);

        if match adjacent.w {
            Some(CellType::Vertex(v)) => v == cell_id_2,
            _ => false,
        } {
            return true;
        }
        if match adjacent.nw {
            Some(CellType::Vertex(v)) => v == cell_id_2,
            _ => false,
        } {
            return true;
        }
        if match adjacent.ne {
            Some(CellType::Vertex(v)) => v == cell_id_2,
            _ => false,
        } {
            return true;
        }
        if match adjacent.e {
            Some(CellType::Vertex(v)) => v == cell_id_2,
            _ => false,
        } {
            return true;
        }
        if match adjacent.se {
            Some(CellType::Vertex(v)) => v == cell_id_2,
            _ => false,
        } {
            return true;
        }
        if match adjacent.sw {
            Some(CellType::Vertex(v)) => v == cell_id_2,
            _ => false,
        } {
            return true;
        }

        false
    }

    /// Iterate over the vertex matrix.
    ///
    /// # Example:
    ///
    /// ```
    /// for (x, y, t) in self
    ///     .vertexes
    ///     .iter()
    ///     .filter(|v| matches!(v.2, vertexes::CellType::Vertex(_)))
    /// {
    ///     match t {
    ///         vertexes::CellType::Vertex(a) => println!("{x},{y} -> Cell({a})"),
    ///         vertexes::CellType::Logo => println!("{x},{y} -> Logo"),
    ///         vertexes::CellType::Background => println!("{x},{y} -> Background"),
    ///     }
    /// }
    /// ```
    pub fn iter(&self) -> VertexesIterator<'_> {
        VertexesIterator {
            vertexes: self,
            x: 0,
            y: 0,
        }
    }
}

/// Iterator for the vertex matrix.
pub struct VertexesIterator<'a> {
    vertexes: &'a Vertexes,
    x: usize,
    y: usize,
}

/// Iterator implementation.
impl Iterator for VertexesIterator<'_> {
    type Item = (usize, usize, CellType);

    fn next(&mut self) -> Option<Self::Item> {
        if self.x >= self.vertexes.width {
            self.x = 0;
            self.y += 1;
            if self.y >= self.vertexes.height {
                return None;
            }
        }

        let result: (usize, usize, CellType) =
            (self.x, self.y, self.vertexes.get_cell(self.x, self.y));
        self.x += 1;
        Some(result)
    }
}
