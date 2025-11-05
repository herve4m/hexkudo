/*
generator.rs

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

//! Manage puzzles and generate random puzzle paths.
//!
//! [`puzzles::Puzzle`] objects represents a puzzle and its parameters, such as the name,
//! difficulty, and colors.
//! A list of [`puzzles::Puzzle`] objects is provided at built time.
//!
//! Before a [`puzzles::Puzzle`] object can be used, its internal representation must be build.
//! The [`puzzles::Puzzle::matrix`] parameter points to a [`puzzle_parse::PuzzleParse`] object that
//! stores an internal representation of the puzzle.
//! This internal representation is build by using the [`puzzle_parse::PuzzleParse::build_edges`]
//! method.
//!
//! To play, a random game must be created.
//! A game is composed of two parts:
//!
//! * A random path represented by a [`path::Path`] object.
//!   You create this object by creating a [`random_path::RandomPath`] object and by using its
//!   [`random_path::RandomPath::generate`] method.
//!   If it takes too long to generate the path, then the method returns an error.
//!   In that case puzzles comes with a list of predefined games that can be used.
//!
//! * A list of diamonds and map represented by a [`diamond_and_map::DiamondAndMap`] object.
//!   The map is the list of the cells with a value that are provided from the beginning of the
//!   game.
//!   They are hints for the player.
//!   You create a [`diamond_and_map::DiamondAndMap`] object by creating a [`diamonds::Diamond`]
//!   object and by using its [`diamonds::Diamond::generate_diamonds`] method.
//!   If it takes too long to generate diamonds, then the method returns an error.
//!   In that case puzzles comes with a list of predefined games that can be used.

pub mod diamond_and_map;
pub mod diamonds;
pub mod edges;
pub mod path;
pub mod puzzle_parse;
pub mod puzzles;
pub mod random_path;
pub mod vertexes;
