/*
path.rs

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

//! Path in the hexkudo graph.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Path object.
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Path {
    /// Path as an ordered list of vertexes.
    path: Vec<usize>,

    /// Stores the visited status of the vertex.
    /// Instead of looking for the vertex in the [`Path::path`] vector, this
    /// [`std::collections::HashSet`] speeds up the lookup.
    visited: HashSet<usize>,
}

impl PartialEq for Path {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl Path {
    /// Create a [`Path`] object.
    pub fn new(num_vertexes: usize) -> Self {
        Self {
            path: Vec::with_capacity(num_vertexes),
            visited: HashSet::with_capacity(num_vertexes),
        }
    }

    /// Create a [`Path`] object from a vector.
    pub fn from_vec(path: &[u8]) -> Self {
        Self {
            path: path.iter().map(|v| *v as usize).collect(),
            visited: HashSet::with_capacity(path.len()),
        }
    }

    /// Remove all the vertexes from the path.
    pub fn clear(&mut self) {
        self.path.clear();
        self.visited.clear();
    }

    /// Add a vertex to the path.
    pub fn push(&mut self, vertex: usize) {
        self.path.push(vertex);
        self.visited.insert(vertex);
    }

    /// Remove the last vertex from the path.
    pub fn pop(&mut self) {
        if let Some(v) = self.path.pop() {
            self.visited.remove(&v);
        }
    }

    /// Get the number of vertexes in the path.
    pub fn len(&self) -> usize {
        self.path.len()
    }

    /// Whether the vertex is in the path or not.
    pub fn contains(&self, vertex: usize) -> bool {
        self.visited.contains(&vertex)
    }

    /// Return a reference to the path vector.
    pub fn get(&self) -> &Vec<usize> {
        &self.path
    }

    /// Return the position of the given vertex in the path. Add one to the return value to get
    /// the cell value.
    pub fn vertex_index(&self, vertex: usize) -> Option<usize> {
        self.path.iter().position(|v| *v == vertex)
    }

    /// Return the first vertex in the path.
    pub fn get_first(&self) -> Option<usize> {
        if self.path.is_empty() {
            None
        } else {
            Some(self.path[0])
        }
    }

    /// Return the last vertex in the path.
    pub fn get_last(&self) -> Option<usize> {
        let l: usize = self.path.len();
        if l > 0 { Some(self.path[l - 1]) } else { None }
    }

    /// Return the vertex ID for the given cell value (cell values start from 1).
    pub fn get_vertex_from_value(&self, value: usize) -> Option<usize> {
        if self.len() < value || value == 0 {
            None
        } else {
            Some(self.path[value - 1])
        }
    }
}
