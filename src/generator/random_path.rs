/*
random_path.rs

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

//! Generate a random path.

use log::debug;
use rand::Rng;
use rand::seq::SliceRandom;
use std::time::Instant;

use super::edges;
use super::path;
use super::vertexes;

// Max duration for trying to find a path, otherwise an error is raised. For large puzzle, it might
// take too long to find a path. In that case a precomputed puzzle is used.
const MAX_TIME_SEC: u64 = 6;

/// Type of errors.
#[derive(Debug, PartialEq)]
pub enum RandomPathError {
    /// No possible path.
    NoPath,

    /// No path found before the timeout.
    DurationExceeded,
}

/// [`RandomPath`] object.
pub struct RandomPath {
    /// Starting vertex.
    /// If [`RandomPath::required_starting_vertex`] is provided, then use that value for the
    /// starting vertex. Otherwise, a random vertex is selected.
    pub starting_vertex: usize,

    /// Number of vertexes in the graph.
    pub num_vertexes: usize,

    /// Graph edges.
    pub edges: edges::Edges,

    /// Number of iterations it took to generate the last random path.
    pub iteration: usize,

    /// Duration in seconds it took to generate the last random path.
    pub duration: f32,

    /// Time when the path generation started. Used to compute the [`RandomPath::duration`].
    start: Instant,

    /// Starting vertex, if the puzzle requires one.
    required_starting_vertex: Option<usize>,
}

impl RandomPath {
    /// Create the object.
    pub fn new(edges: &edges::Edges, vertexes: &vertexes::Vertexes) -> Self {
        Self {
            starting_vertex: 0,
            num_vertexes: vertexes.num_vertexes,
            edges: edges.clone(),
            iteration: 0,
            duration: 0.0,
            start: Instant::now(),
            required_starting_vertex: vertexes.required_starting_vertex,
        }
    }

    /// Generate and return a random path.
    ///
    /// If a starting vertex is provided in `starting_vertex`, then it is used only if puzzle does
    /// not require one.
    ///
    /// # Errors
    ///
    /// The method returns an error if a path cannot be found (this is a design error in the
    /// provided puzzle), or if it takes too long to produce a path. In that later case, the
    /// method can be retried.
    pub fn generate(
        &mut self,
        starting_vertex: Option<usize>,
    ) -> Result<path::Path, RandomPathError> {
        self.iteration = 0;
        self.duration = 0.0;
        self.start = Instant::now();

        // If the required starting vertex is defined, then use that.
        // Otherwise use the starting_vertex parameter if it's defined.
        // Otherwise use a random vertex.
        self.starting_vertex = match self.required_starting_vertex {
            Some(v) => v,
            None => match starting_vertex {
                Some(v) => {
                    if v > self.num_vertexes {
                        self.num_vertexes - 1
                    } else {
                        v
                    }
                }
                None => rand::rng().random_range(0..self.num_vertexes),
            },
        };

        debug!(
            "Starting vertex = {}  Number of vertexes = {}",
            self.starting_vertex, self.num_vertexes
        );

        let mut path: path::Path = path::Path::new(self.num_vertexes);
        let res: Result<(), RandomPathError> =
            self.find_path(self.starting_vertex, &mut self.edges.clone(), &mut path);
        self.duration = self.start.elapsed().as_secs_f32();
        debug!(
            "Iterations = {}  Duration = {}",
            self.iteration, self.duration
        );
        match res {
            Err(e) => Err(e),
            Ok(()) => Ok(path),
        }
    }

    /// Recursively find a path.
    fn find_path(
        &mut self,
        current_vertex: usize,
        edges: &mut edges::Edges,
        path: &mut path::Path,
    ) -> Result<(), RandomPathError> {
        debug!(
            "== Going to vertex {} (iteration {})",
            current_vertex, self.iteration
        );
        if path.contains(current_vertex) {
            debug!("    Back: vertex already in path");
            return Err(RandomPathError::NoPath);
        }
        path.push(current_vertex);
        if path.len() == self.num_vertexes {
            return Ok(());
        }

        self.iteration += 1;
        if self.start.elapsed().as_secs() >= MAX_TIME_SEC {
            return Err(RandomPathError::DurationExceeded);
        }

        // Verify quickly if there is a required edge
        if let Some(v2) = edges
            .get_vertexes(current_vertex, edges::EdgeStatus::Required)
            .iter()
            .find(|&vertex| !path.contains(*vertex))
        {
            // Recursively propagate the status of the edge
            if !self.set_status_adjacent(current_vertex, edges) {
                debug!("    Back: the edge {current_vertex}-{v2} is not valid 1");
                path.pop();
                return Err(RandomPathError::NoPath);
            }
            // Follow the required edge
            match self.find_path(*v2, edges, path) {
                Ok(()) => return Ok(()),
                Err(e) => {
                    if e == RandomPathError::NoPath {
                        debug!("    Back: the edge {current_vertex}-{v2} is not valid 2");
                        path.pop();
                    }
                    return Err(e);
                }
            }
        }

        // Randomize the order in which to test the edges
        let mut indices: Vec<usize> = edges
            .get_vertexes(current_vertex, edges::EdgeStatus::Undecided)
            .iter()
            .filter(|&vertex| !path.contains(*vertex))
            .copied()
            .collect();
        indices.shuffle(&mut rand::rng());

        for v2 in indices {
            debug!("    Selecting edge {current_vertex}-{v2}");

            let mut new_edges: edges::Edges = edges.clone();
            new_edges.set_status(current_vertex, v2, edges::EdgeStatus::Required);
            edges.set_status(current_vertex, v2, edges::EdgeStatus::Deleted);

            // Recursively propagate the new status of the edge
            if !self.set_status_adjacent(current_vertex, &mut new_edges) {
                continue;
            }

            // Follow the selected edge
            match self.find_path(v2, &mut new_edges, path) {
                Ok(()) => return Ok(()),
                Err(e) => {
                    if e == RandomPathError::DurationExceeded {
                        return Err(e);
                    }
                }
            }
        }
        debug!("    Back: no eligible edge");
        path.pop();
        Err(RandomPathError::NoPath)
    }

    /// Recursively propagate the status of the edges from the given vertex.
    ///
    /// # Errors
    ///
    /// Return `false` if the status cannot be set, because a loop is detected for example.
    fn set_status_adjacent(&self, v1: usize, edges: &mut edges::Edges) -> bool {
        let mut vertexes_to_update: Vec<usize> = Vec::with_capacity(self.num_vertexes);

        // For the starting vertex, keep the required edge, but remove all the other edges
        if self.starting_vertex == v1 {
            for v2 in edges.get_vertexes(v1, edges::EdgeStatus::Undecided) {
                edges.set_status(v1, v2, edges::EdgeStatus::Deleted);
                vertexes_to_update.push(v2);
                debug!("    Edge {v1}-{v2} deleted");
            }
            for v in &vertexes_to_update {
                if !self.set_status_adjacent(*v, edges) {
                    return false;
                }
            }
            return true;
        }

        // Count the number of total and required edges for the vertex
        let num_required: usize = edges.num_status(v1, edges::EdgeStatus::Required);
        let num_edges: usize = edges.num_edges(v1);

        // Too many required edges
        if num_required > 2 {
            debug!("    Vertex {v1} has too many ({num_required}) required edges");
            return false;
        }

        // No more edges for this vertex
        if num_edges == 0 {
            debug!("    Vertex {v1} has no edges");
            return false;
        }

        // Two required edges, therefore the other edges get deleted
        if num_required == 2 {
            for v2 in edges.get_vertexes(v1, edges::EdgeStatus::Undecided) {
                edges.set_status(v1, v2, edges::EdgeStatus::Deleted);
                vertexes_to_update.push(v2);
                debug!("    Edge {v1}-{v2} deleted");
            }
        }
        // If the vertex has two edges or less, then all are required
        else if num_edges <= 2 {
            for v2 in edges.get_vertexes(v1, edges::EdgeStatus::Undecided) {
                edges.set_status(v1, v2, edges::EdgeStatus::Required);
                vertexes_to_update.push(v2);
                // Verify that if by adding this required edge, a loop is not formed
                if Self::is_loop(v1, v2, edges, v1) {
                    return false;
                }
                debug!("    Edge {v1}-{v2} required");
            }
        }

        for v in &vertexes_to_update {
            if !self.set_status_adjacent(*v, edges) {
                return false;
            }
        }
        true
    }

    /// Detect a loop in the path.
    fn is_loop(
        previous_vertex: usize,
        vertex: usize,
        edges: &edges::Edges,
        start_vertex: usize,
    ) -> bool {
        for v2 in edges
            .get_vertexes(vertex, edges::EdgeStatus::Required)
            .iter()
            .filter(|&v| *v != previous_vertex)
        {
            if *v2 == start_vertex {
                debug!("    Loop detected from vertex {start_vertex}");
                return true;
            }
            if Self::is_loop(vertex, *v2, edges, start_vertex) {
                return true;
            }
        }
        false
    }
}
