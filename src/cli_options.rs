/*
cli_options.rs

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

//! Process command-line options.
//!
//! These options are intended for developers creating puzzles.
//! In command-line mode, Hexkudo can generate random paths that developers can copy to complete
//! their puzzle file in the `src/generator/puzzles` directory.
//!
//! # Examples
//!
//! List the available puzzles:
//!
//! ```
//! $ flatpak run io.github.herve4m.Hexkudo --ls
//! Classic medium
//! Classic easy
//! Heart easy
//! Square medium
//! Heart medium
//! Heart hard
//! Square easy
//! Square hard
//! Classic hard
//! ```
//!
//! Generate three puzzles for the Classic map at the easy difficulty level:
//!
//! ```
//! $ flatpak run io.github.herve4m.Hexkudo -c 3 -p Classic -f easy
//!
//! pub fn get_random_sample_path() -> puzzles::PuzzleSampleGame {
//!     let sample_path: [[u8; 22]; 3] = [
//!         [3, 6, 7, 8, 4, 1, 0, 2, 5, 9, 13, 10, 14, 17, 18, 20, 21, 19, 15, 11, 12, 16],
//!         [8, 7, 4, 1, 3, 0, 2, 6, 10, 5, 9, 13, 17, 20, 21, 19, 18, 14, 15, 11, 12, 16],
//!         [13, 17, 18, 20, 21, 19, 15, 14, 10, 9, 5, 2, 6, 3, 0, 1, 4, 7, 8, 11, 16, 12],
//!     ];
//!     let sample_diamonds: [Vec<(u8, u8)>; 3]  = [
//!         [(18, 20), (9, 13), (17, 18), (13, 10)].to_vec(),
//!         [(18, 14), (19, 18), (6, 10), (0, 2), (3, 0), (5, 9)].to_vec(),
//!         [(5, 2), (20, 21), (4, 7), (7, 8), (18, 20)].to_vec(),
//!     ];
//!     let sample_maps: [Vec<u8>; 3] = [
//!         [3, 6, 8, 16].to_vec(),
//!         [8, 16, 7].to_vec(),
//!         [13, 16, 11, 12].to_vec(),
//!     ];
//!     let i:usize = rand::rng().random_range(0..sample_path.len());
//!     puzzles::PuzzleSampleGame {
//!         path: Vec::from(sample_path[i]),
//!         diamonds: sample_diamonds[i].clone(),
//!         map: sample_maps[i].clone()
//!     }
//! }
//! ```

use clap::Parser;
use log::debug;
use std::collections::HashMap;
use std::env;

use crate::config::COPYRIGHT_NOTICE;
use crate::generator::diamond_and_map;
use crate::generator::diamonds;
use crate::generator::path;
use crate::generator::puzzles;
use crate::generator::random_path;
use crate::generator::vertexes::Vertexes;

/// Build random Hexkudo paths for developers.
#[derive(Parser)]
#[command(about, long_about = None, version, long_version = COPYRIGHT_NOTICE, ignore_errors = true)]
struct Args {
    /// List the puzzles
    #[arg(short, long, default_value_t = false)]
    ls: bool,

    /// Name of the puzzle to generate a path for
    #[arg(short, long, group = "generate")]
    puzzle: Option<String>,

    /// Difficulty level for the puzzle
    #[arg(value_enum, short = 'f', long, default_value_t=puzzles::Difficulty::Medium, requires = "generate")]
    difficulty: puzzles::Difficulty,

    /// Number of paths to generate
    #[arg(short, long, default_value_t = 1, requires = "generate")]
    count: usize,

    /// Print some statistics after generating the paths
    #[arg(short, long, default_value_t = false, requires = "generate")]
    summary: bool,

    /// Enable debug messages
    #[arg(short, long, default_value_t = false)]
    debug: bool,
}

/// Parse and process command-line options.
pub fn parse() -> Option<u8> {
    let args: Args = Args::parse();

    if args.debug {
        println!("DEBUG");
        unsafe {
            env::set_var("RUST_LOG", "debug");
        }
    }
    env_logger::init();

    if !args.ls && args.puzzle.is_none() {
        return None;
    }

    let mut puzzle_hash: HashMap<(String, puzzles::Difficulty), puzzles::Puzzle> =
        puzzles::puzzle_map();

    //
    // List the puzzles
    //
    if args.ls {
        for name_difficulty in puzzle_hash.keys() {
            let (name, difficulty) = name_difficulty;
            match difficulty {
                puzzles::Difficulty::Easy => println!("{name} easy"),
                puzzles::Difficulty::Medium => println!("{name} medium"),
                puzzles::Difficulty::Hard => println!("{name} hard"),
            }
        }
        return Some(0);
    }

    //
    // Parse the definition of the requested puzzle and build its internal representation
    //
    let puzzle_name: String = args.puzzle.expect("Cannot retrieve puzzle name");
    let mut path: random_path::RandomPath;
    let vertexes: &Vertexes;

    match puzzle_hash.get_mut(&(puzzle_name.clone(), args.difficulty)) {
        Some(p) => {
            match p.matrix.build_edges() {
                Ok(()) => (),
                Err(msg) => panic!("Error: {puzzle_name}: {msg}"),
            }
            path = random_path::RandomPath::new(&p.matrix.edges, &p.matrix.vertexes);
            vertexes = &p.matrix.vertexes;
        }
        None => {
            eprintln!(
                "Unknown puzzle {} {:?}. Use --ls to list the available puzzles.",
                puzzle_name, args.difficulty
            );
            return Some(1);
        }
    }

    let mut path_list: Vec<String> = Vec::new();
    let mut map_list: Vec<String> = Vec::new();
    let mut diamond_list: Vec<String> = Vec::new();
    let mut total: f32 = 0.0;
    let mut max: f32 = 0.0;
    let mut errors: usize = 0;
    let mut iterations: usize = 0;
    let mut i: usize = 0;
    while i < args.count {
        debug!("Iteration {i}");

        // Generate the path
        let ret: Result<path::Path, random_path::RandomPathError> = path.generate(None);
        match ret {
            Ok(random_path) => {
                total += path.duration;
                if path.duration > max {
                    max = path.duration;
                }
                iterations += path.iteration;

                // Verify that the path has the expected length
                if random_path.len() != path.num_vertexes {
                    eprintln!(
                        "Wrong length: {} instead of {}: {:?}",
                        random_path.len(),
                        path.num_vertexes,
                        random_path.get()
                    );
                    panic!("Bug: wrong length for the generated path");
                }

                // Verify that there are no duplicated vertexes
                let mut p: Vec<usize> = random_path.get().clone();
                p.sort_unstable();
                p.dedup();
                if p.len() != path.num_vertexes {
                    eprintln!("Duplicated vertexes in path: {:?}", random_path.get());
                    panic!("Bug: duplicated vertexes in generated path");
                }

                // Generate random diamonds and map for this path
                let ret_diamonds: Result<diamond_and_map::DiamondAndMap, diamonds::DiamondError> =
                    diamonds::Diamond::new(&path.edges, &random_path).generate_diamonds(vertexes);
                match ret_diamonds {
                    Ok(diamond_and_map) => {
                        map_list.push(format!("{:?}", diamond_and_map.get_map()));
                        diamond_list.push(format!("{:?}", diamond_and_map.get_diamonds()));
                    }
                    Err(_) => {
                        // It took too long, the diamond and map generating algorithm gave up
                        errors += 1;
                        debug!("ERROR generating random diamonds and map");
                        continue;
                    }
                }

                path_list.push(format!("{:?}", random_path.get()));
                i += 1;
            }

            Err(_) => {
                // It took too long, the path generating algorithm gave up
                errors += 1;
                debug!("ERROR generating random path");
            }
        }
    }

    //
    // Print the Rust code that can be added to a puzzle description as fallback in case generating
    // a random puzzle takes too long.
    //
    let l: usize = path_list.len();
    println!(
        "
/// Return a tuple with the path, the diamond, and the map lists.
pub fn get_random_sample_path() -> puzzles::PuzzleSampleGame {{
    let sample_path: [[u8; {}]; {}] = [",
        path.num_vertexes, l
    );
    for p in &path_list {
        println!("        {p},");
    }

    println!(
        "    ];
    let sample_diamonds: [Vec<(u8, u8)>; {l}]  = [",
    );
    for d in &diamond_list {
        println!("        {d}.to_vec(),");
    }

    println!(
        "    ];
    let sample_maps: [Vec<u8>; {l}] = ["
    );
    for m in &map_list {
        println!("        {m}.to_vec(),");
    }

    println!(
        "    ];
    let i:usize = rand::rng().random_range(0..sample_path.len());
    puzzles::PuzzleSampleGame {{
        path: Vec::from(sample_path[i]),
        diamonds: sample_diamonds[i].clone(),
        map: sample_maps[i].clone()
    }}
}}"
    );

    // Print some stats
    if args.summary {
        println!(
            "
        total time = {}s
      average time = {}s
          max time = {}s
average iterations = {}
            errors = {}",
            total,
            total / args.count as f32,
            max,
            iterations / args.count,
            errors
        );
    }
    Some(0)
}
