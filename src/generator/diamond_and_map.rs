/*
diamond_and_map.rs

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

//! Manage and compute diamonds and maps.
//!
//! A diamond is a marking between two cells that indicates consecutive
//! numbers.
//! A map is a cell where the number is already provided at the beginning
//! of the puzzle. It provides a hint to the user.

use std::collections::HashSet;
use std::hash::{Hash, Hasher};

use super::vertexes;

/// Diamond representation.
#[derive(Debug, Default, Clone)]
struct Diamond {
    vertex1: usize,
    vertex2: usize,
}

impl Hash for Diamond {
    fn hash<H: Hasher>(&self, state: &mut H) {
        if self.vertex1 < self.vertex2 {
            self.vertex1.hash(state);
            self.vertex2.hash(state);
        } else {
            self.vertex2.hash(state);
            self.vertex1.hash(state);
        }
    }
}

impl PartialEq for Diamond {
    fn eq(&self, other: &Self) -> bool {
        (self.vertex1 == other.vertex1 && self.vertex2 == other.vertex2)
            || (self.vertex1 == other.vertex2 && self.vertex2 == other.vertex1)
    }
}

impl Eq for Diamond {}

impl Diamond {
    /// Whether the given vertex is one member of the diamond.
    fn is_in(&self, vertex: usize) -> bool {
        self.vertex1 == vertex || self.vertex2 == vertex
    }

    /// Given a vertex, return the other vertex.
    fn other(&self, vertex: usize) -> usize {
        if self.vertex1 == vertex {
            self.vertex2
        } else {
            self.vertex1
        }
    }
}

/// Manage diamonds and maps.
#[derive(Debug, Default, Clone)]
pub struct DiamondAndMap {
    /// List of diamonds.
    diamonds: HashSet<Diamond>,

    /// List of hints.
    maps: HashSet<usize>,

    /// Number of vertexes in the puzzle.
    num_vertexes: usize,

    /// Starting vertex
    starting_vertex: usize,

    /// Final vertex.
    ending_vertex: usize,
}

impl DiamondAndMap {
    /// Create a [`DiamondAndMap`] object.
    pub fn new(num_vertexes: usize, starting_vertex: usize, ending_vertex: usize) -> Self {
        Self {
            diamonds: HashSet::new(),
            maps: HashSet::with_capacity(num_vertexes),
            num_vertexes,
            starting_vertex,
            ending_vertex,
        }
    }

    /// Create a [`DiamondAndMap`] object from the provided diamond and map lists.
    pub fn from_vec(
        diamond_list: &Vec<(u8, u8)>,
        map_list: &Vec<u8>,
        num_vertexes: usize,
        starting_vertex: usize,
        ending_vertex: usize,
    ) -> Self {
        let mut obj: DiamondAndMap =
            DiamondAndMap::new(num_vertexes, starting_vertex, ending_vertex);
        for (vertex1, vertex2) in diamond_list {
            obj.insert(*vertex1 as usize, *vertex2 as usize);
        }
        for m in map_list {
            obj.maps.insert(*m as usize);
        }
        obj
    }

    /// Remove all the diamonds.
    pub fn clear(&mut self) {
        self.diamonds.clear();
    }

    /// Add a diamond to the object.
    pub fn insert(&mut self, vertex1: usize, vertex2: usize) {
        self.diamonds.insert(Diamond { vertex1, vertex2 });
    }

    /// Remove a diamond from the object.
    pub fn remove(&mut self, vertex1: usize, vertex2: usize) {
        self.diamonds.remove(&Diamond { vertex1, vertex2 });
    }

    /// Build the map list (hints) from the diamond list.
    pub fn compute(&mut self, vertexes: &vertexes::Vertexes) {
        self.maps.clear();

        // The starting and ending vertexes are always mapped
        self.maps.insert(self.starting_vertex);
        self.maps.insert(self.ending_vertex);

        // When a vertex has two opposite diamonds, then remove the diamonds and map the
        // destination vertexes.
        //
        //  / \    / \    / \           / \    / \    / \
        // |   |<>|   |<>|   |   ===>  | 1 |  |   |  | 3 |
        //  \ /    \ /    \ /           \ /    \ /    \ /
        //
        for vertex in 0..self.num_vertexes {
            // Retrieve the destination vertexes for `vertex`
            let vertexes_to_map: Vec<usize> = self
                .diamonds
                .iter()
                .filter(|d| d.is_in(vertex))
                .map(|d| d.other(vertex))
                .collect();

            // If the vertex has two diamonds and these diamonds are opposite, then map the
            // destination vertexes and remove the diamonds
            if vertexes_to_map.len() == 2
                && vertexes
                    .get_adjacent(vertex)
                    .opposite(vertexes_to_map[0], vertexes_to_map[1])
            {
                for v in vertexes_to_map {
                    self.maps.insert(v);
                    self.remove(vertex, v);
                }
            }
        }

        // Mapped vertexes should not have diamonds. If a map vertex has a diamond, then remove
        // the diamond, and map the destination vertex.
        //
        //  / \    / \           / \    / \
        // | 4 |<>|   |   ===>  | 4 |  | 5 |
        //  \ /    \ /           \ /    \ /
        //
        let mut map_added: usize;
        loop {
            let mapped_vertexes: Vec<usize> = self.maps.iter().copied().collect();
            map_added = 0;
            for mapped_vertex in mapped_vertexes {
                let vertexes: Vec<usize> = self
                    .diamonds
                    .iter()
                    .filter(|d| d.is_in(mapped_vertex))
                    .map(|d| d.other(mapped_vertex))
                    .collect();
                for v in vertexes {
                    self.maps.insert(v);
                    map_added += 1;
                    self.remove(mapped_vertex, v);
                }
            }
            if map_added == 0 {
                break;
            }
        }
    }

    /// Return the list of maps (hints). Require that you run `compute()` before, otherwise the
    /// list is empty.
    pub fn get_map(&self) -> Vec<usize> {
        self.maps.iter().copied().collect()
    }

    /// Return the list of diamonds.
    pub fn get_diamonds(&self) -> Vec<(usize, usize)> {
        self.diamonds
            .iter()
            .map(|d| (d.vertex1, d.vertex2))
            .collect()
    }

    /// Return the list of diamonds and the list of maps.
    pub fn get_diamond_and_map(&self) -> (Vec<(usize, usize)>, Vec<usize>) {
        (self.get_diamonds(), self.get_map())
    }
}
