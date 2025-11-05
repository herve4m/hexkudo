/*
diamonds.rs

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

//! Generate diamonds and maps.
//!
//! A diamond is a marking between two cells that indicates consecutive
//! numbers.
//! A map is a cell where the number is already provided at the beginning
//! of the puzzle. It provides a hint to the user.

use log::{Level, debug, log_enabled};
use rand::seq::SliceRandom;
use std::time::Instant;

use super::diamond_and_map;
use super::edges;
use super::path;
use super::vertexes;

// Max duration for trying to find an alternate path, otherwise an error is raised.
// For large puzzle, it might take too long to find alternate paths. In that case a precomputed
// puzzle is used.
const MAX_TIME_SEC: u64 = 6;

/// Type of errors.
#[derive(Debug, PartialEq)]
pub enum DiamondError {
    /// No possible path.
    NoPath,

    /// No path found before the timeout.
    DurationExceeded,
}

/// Diamond object.
pub struct Diamond<'a> {
    /// Puzzle's edges.
    edges: edges::Edges,

    /// Puzzle path (solution).
    path: &'a path::Path,

    /// Number of vertexes in the graph.
    num_vertexes: usize,

    /// Starting vertex.
    starting_vertex: usize,

    /// Last vertex.
    ending_vertex: usize,

    /// Temporary working alternate path.
    wpath: path::Path,

    /// Number of iterations it took to generate diamonds and paths.
    pub iteration: usize,

    /// Duration in seconds it took to generate diamonds and paths.
    pub duration: f32,

    /// Time when the alternate path research started. Used to compute the [`Diamond::duration`].
    start: Instant,
}

impl<'a> Diamond<'a> {
    /// Create the object.
    pub fn new(edges: &edges::Edges, path: &'a path::Path) -> Self {
        let num_vertexes: usize = path.len();
        let mut e: edges::Edges = edges.clone();
        let mut i: usize = 0;

        // As a starting point, make all the edges along the path required. When generating a
        // unique path, one edge is made undecided per iteration, and an alternate path is searched.
        while i < num_vertexes - 1 {
            let vertex1: usize = path.get()[i];
            let vertex2: usize = path.get()[i + 1];

            e.set_status(vertex1, vertex2, edges::EdgeStatus::Required);
            i += 1;
        }

        Self {
            edges: e,
            path,
            num_vertexes,
            starting_vertex: path.get()[0],
            ending_vertex: *path.get().last().unwrap_or(&0),
            wpath: path::Path::new(num_vertexes),
            iteration: 0,
            duration: 0.0,
            start: Instant::now(),
        }
    }

    /// Generate and return diamonds and maps.
    pub fn generate_diamonds(
        &mut self,
        vertexes: &vertexes::Vertexes,
    ) -> Result<diamond_and_map::DiamondAndMap, DiamondError> {
        // Store required diamonds
        let mut diamond_and_map: diamond_and_map::DiamondAndMap =
            diamond_and_map::DiamondAndMap::new(
                self.num_vertexes,
                self.starting_vertex,
                self.ending_vertex,
            );

        // Store deleted diamonds
        let mut deleted_diamonds: Vec<(usize, usize)> = Vec::with_capacity(self.num_vertexes);

        // Create a random list of diamonds. When starting, all the diamonds for all the edges
        // exist. Then, in each iteration, a diamond is removed and an alternate path is searched.
        // If such path exists, then the diamond is required, otherwise it is dropped.
        let mut diamonds: Vec<usize> = Vec::from_iter(0..self.num_vertexes - 1);
        diamonds.shuffle(&mut rand::rng());

        if log_enabled!(Level::Debug) {
            debug!("Finding unique path");
            debug!("    source path = {:?}", self.path.get());
            debug!("          start = {:?}", self.starting_vertex);
            debug!("            end = {:?}", self.ending_vertex);
            debug!("       diamonds = {diamonds:?}");
        }

        self.iteration = 0;
        self.duration = 0.0;
        self.start = Instant::now();

        for d in diamonds {
            let mut e: edges::Edges = self.edges.clone();
            let vertex1: usize = self.path.get()[d];
            let vertex2: usize = self.path.get()[d + 1];

            debug!("=== deleting diamond {vertex1} <> {vertex2}");
            deleted_diamonds.push((vertex1, vertex2));

            // Synchronize the deleted diamonds with the edge list: for deleted diamonds, the
            // corresponding edges are changed from required to undecided.
            for (v1, v2) in deleted_diamonds.iter_mut() {
                e.set_status(*v1, *v2, edges::EdgeStatus::Undecided);
            }

            // Propagate the changes in the edge list to speed up the alternate path search
            for v2 in self.path.get() {
                self.set_status_adjacent(*v2, &mut e);
            }

            self.wpath.clear();
            // Search for an alternate path
            match self.is_there_another_path(self.starting_vertex, &mut e) {
                Ok(()) => {
                    debug!("    requiring diamond {vertex1} <> {vertex2}");
                    self.edges
                        .set_status(vertex1, vertex2, edges::EdgeStatus::Required);
                    deleted_diamonds.pop();
                    diamond_and_map.insert(vertex1, vertex2);
                }
                Err(e) => {
                    if e == DiamondError::DurationExceeded {
                        self.duration = self.start.elapsed().as_secs_f32();
                        return Err(e);
                    }
                }
            }
        }
        self.duration = self.start.elapsed().as_secs_f32();
        debug!(
            "Iterations = {}  Duration = {}",
            self.iteration, self.duration
        );
        diamond_and_map.compute(vertexes);
        Ok(diamond_and_map)
    }

    // Search for an alternate path.
    fn is_there_another_path(
        &mut self,
        current_vertex: usize,
        edges: &mut edges::Edges,
    ) -> Result<(), DiamondError> {
        debug!(
            "== Going to vertex {} (iteration {})",
            current_vertex, self.iteration
        );
        if self.wpath.contains(current_vertex) {
            debug!("    Back: vertex already in path");
            return Err(DiamondError::NoPath);
        }
        self.wpath.push(current_vertex);

        // The end vertex has been reached
        if current_vertex == self.ending_vertex {
            if log_enabled!(Level::Debug) {
                debug!("   End reached");
                debug!("     current path = {:?}", self.wpath.get());
                debug!("      source path = {:?}", self.path.get());
            }
            if self.wpath.len() != self.num_vertexes {
                debug!("   the sizes are different");
                self.wpath.pop();
                return Err(DiamondError::NoPath);
            }
            if self.wpath == *self.path {
                debug!("   the paths are equal");
                self.wpath.pop();
                return Err(DiamondError::NoPath);
            }
            // An alternate path exists
            debug!("   the paths have the same size but are different: alternate path found");
            return Ok(());
        }

        self.iteration += 1;
        if self.start.elapsed().as_secs() >= MAX_TIME_SEC {
            return Err(DiamondError::DurationExceeded);
        }

        // Verify quickly if there is a required edge
        if let Some(v2) = edges
            .get_vertexes(current_vertex, edges::EdgeStatus::Required)
            .iter()
            .find(|&vertex| !self.wpath.contains(*vertex))
        {
            if !self.set_status_adjacent(current_vertex, edges) {
                debug!("   Back: the edge {current_vertex}-{v2} is not valid 1");
                self.wpath.pop();
                return Err(DiamondError::NoPath);
            }

            debug!("   Edge {current_vertex}-{v2}: following diamond");
            match self.is_there_another_path(*v2, edges) {
                Ok(()) => return Ok(()),
                Err(e) => {
                    if e == DiamondError::NoPath {
                        debug!(
                            "   Back: the edge {current_vertex}-{v2} is not valid: following diamond failed"
                        );
                        self.wpath.pop();
                    }
                    return Err(e);
                }
            }
        }

        // List of possible next vertexes
        let vertexes: Vec<usize> = edges
            .get_vertexes(current_vertex, edges::EdgeStatus::Undecided)
            .iter()
            .filter(|&vertex| !self.wpath.contains(*vertex))
            .copied()
            .collect();
        for v2 in vertexes {
            debug!("   Selecting edge {current_vertex}-{v2}");
            let mut new_edges: edges::Edges = edges.clone();
            new_edges.set_status(current_vertex, v2, edges::EdgeStatus::Required);
            edges.set_status(current_vertex, v2, edges::EdgeStatus::Deleted);

            // Recursively propagate the new status of the edge
            if !self.set_status_adjacent(current_vertex, &mut new_edges) {
                continue;
            }

            // Follow the selected edge
            match self.is_there_another_path(v2, &mut new_edges) {
                Ok(()) => return Ok(()),
                Err(e) => {
                    if e == DiamondError::DurationExceeded {
                        return Err(e);
                    }
                }
            }
            debug!(
                "   Continue with the next edge, because the edge {current_vertex}-{v2} is not valid"
            );
        }
        self.wpath.pop();
        Err(DiamondError::NoPath)
    }

    /// Recursively propagate the status of the edges from the given vertex.
    ///
    /// # Errors
    ///
    /// Return `false` if the status cannot be set, because a vertex has too many required edges
    /// for example.
    fn set_status_adjacent(&self, v1: usize, edges: &mut edges::Edges) -> bool {
        let mut vertexes_to_update: Vec<usize> = Vec::with_capacity(self.num_vertexes);

        // For the starting vertex, keep the required edge, but remove all the other edges
        if self.starting_vertex == v1 || self.ending_vertex == v1 {
            // Ensure that at least one edge is required
            if !edges
                .get_vertexes(v1, edges::EdgeStatus::Required)
                .is_empty()
            {
                for v2 in edges.get_vertexes(v1, edges::EdgeStatus::Undecided) {
                    edges.set_status(v1, v2, edges::EdgeStatus::Deleted);
                    vertexes_to_update.push(v2);
                    debug!("   Edge {v1}-{v2} deleted");
                }
                for v in &vertexes_to_update {
                    if !self.set_status_adjacent(*v, edges) {
                        return false;
                    }
                }
            }
            return true;
        }

        // Count the number of total and required edges for the vertex
        let num_required: usize = edges.num_status(v1, edges::EdgeStatus::Required);
        let num_edges: usize = edges.num_edges(v1);

        // Too many required edges
        if num_required > 2 {
            debug!("   Vertex {v1} has too many ({num_required}) required edges");
            return false;
        }

        // No more edges for this vertex
        if num_edges == 0 {
            debug!("   Vertex {v1} has no edges");
            return false;
        }

        // Two required edges, therefore the other edges get deleted
        if num_required == 2 {
            for v2 in edges.get_vertexes(v1, edges::EdgeStatus::Undecided) {
                edges.set_status(v1, v2, edges::EdgeStatus::Deleted);
                vertexes_to_update.push(v2);
                debug!("   Edge {v1}-{v2} deleted");
            }
        }
        // If the vertex has two edges or less, then all are required
        else if num_edges <= 2 {
            for v2 in edges.get_vertexes(v1, edges::EdgeStatus::Undecided) {
                edges.set_status(v1, v2, edges::EdgeStatus::Required);
                vertexes_to_update.push(v2);
                debug!("   Edge {v1}-{v2} required");
            }
        }

        for v in &vertexes_to_update {
            if !self.set_status_adjacent(*v, edges) {
                return false;
            }
        }
        true
    }
}
