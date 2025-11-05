/*
edges.rs

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

//! Edges between vertexes in the Hexkudo graph.

use log::debug;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Status of the edges in the Hexkudo graph.
///
/// - an `Undecided` edge is an edge for which the path finder has not decided yet if it is
///   required or impossible (deleted)
/// - a `Required` edge indicates a path that must be used.
/// - a `Deleted` edge is an impossible path; similar to removing the edge.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
pub enum EdgeStatus {
    Undecided,
    Required,
    Deleted,
}

/// Represent the edges in the Hexkudo graph.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Edges {
    /// For each vertex, the [`std::collections::HashMap`] stores the list of the adjacent vertexes.
    ///
    /// With each adjacent vertex, the [`std::collections::HashMap`] stores the status of the edge.
    /// The status specifies if the edge is required, undecided, or impossible (deleted).
    edges: HashMap<usize, Vec<(usize, EdgeStatus)>>,
}

impl Default for Edges {
    fn default() -> Self {
        Self::new()
    }
}

impl Edges {
    /// Create the edge object that stores all the edges.
    pub fn new() -> Self {
        Self {
            edges: HashMap::new(),
        }
    }

    /// Remove all the edges from the object.
    pub fn clear(&mut self) {
        self.edges.clear();
    }

    /// Add all the adjacent vertexes of the given vertex.
    pub fn push_from_array(
        &mut self,
        vertex: usize,
        adjacent_vertex_array: &[usize],
        status: EdgeStatus,
    ) {
        // Because the method is used in a loop to initialize all the edges, it is not necessary to
        // create the edges in both directions.
        self.edges.insert(
            vertex,
            adjacent_vertex_array.iter().map(|v| (*v, status)).collect(),
        );
    }

    /// For the given vertex, return all the adjacent vertexes with the given status.
    pub fn get_vertexes(&self, vertex: usize, status: EdgeStatus) -> Vec<usize> {
        match self.edges.get(&vertex) {
            Some(a) => a.iter().filter(|t| t.1 == status).map(|t| t.0).collect(),
            None => Vec::new(),
        }
    }

    /// For the given vertex, return all the adjacent vertexes with the required or undecided
    /// status.
    pub fn get_not_deleted_vertexes(&self, vertex: usize) -> Vec<usize> {
        match self.edges.get(&vertex) {
            Some(a) => a
                .iter()
                .filter(|t| t.1 != EdgeStatus::Deleted)
                .map(|t| t.0)
                .collect(),
            None => Vec::new(),
        }
    }

    /// Set the status of the edge between the given vertexes.
    pub fn set_status(&mut self, vertex1: usize, vertex2: usize, status: EdgeStatus) {
        if let Some(a) = self.edges.get_mut(&vertex1) {
            for t in a.iter_mut().filter(|t| t.0 == vertex2) {
                t.1 = status;
            }
        }

        if let Some(a) = self.edges.get_mut(&vertex2) {
            for t in a.iter_mut().filter(|t| t.0 == vertex1) {
                t.1 = status;
            }
        }
    }

    /// For the given vertex, return the number of adjacent vertexes with the given status.
    pub fn num_status(&self, vertex: usize, status: EdgeStatus) -> usize {
        match self.edges.get(&vertex) {
            Some(a) => a.iter().filter(|t| t.1 == status).count(),
            None => 0,
        }
    }

    /// Number of adjacent (non-deleted) vertexes of the given vertex.
    pub fn num_edges(&self, vertex: usize) -> usize {
        match self.edges.get(&vertex) {
            Some(a) => a.iter().filter(|t| t.1 != EdgeStatus::Deleted).count(),
            None => 0,
        }
    }

    /// Print the edges.
    pub fn debug(&self) {
        let mut s: String = String::new();
        let mut v: Vec<_> = self.edges.iter().collect();

        v.sort_by_key(|a| a.0);
        for (v1, e) in v {
            s.clear();
            s.push_str(&format!("{v1:>3} -->"));
            for (v2, c) in e {
                match c {
                    EdgeStatus::Required => s.push_str(&format!(" {v2}(required)")),
                    EdgeStatus::Undecided => s.push_str(&format!(" {v2}")),
                    EdgeStatus::Deleted => (),
                }
            }
            debug!("{s}");
        }
    }
}
