/*
Based on GNOME Sudoku at https://gitlab.gnome.org/GNOME/gnome-sudoku

print_job.rs

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

//! `GtkPrintOperation` object to print puzzles.

use gettextrs::gettext;
use log::{Level, debug, log_enabled};

use adw::prelude::*;
use gtk::cairo::{Context, Surface, TextExtents};
use gtk::glib;
use gtk::subclass::prelude::*;

use crate::draw;
use crate::generator::path;
use crate::generator::puzzles;

/// Print parameters
#[derive(Debug)]
pub struct PrintJobParameters {
    /// `GtkWindow` required to run the print operation.
    pub window: gtk::Window,

    /// [`puzzles::Puzzle`] object to print.
    pub puzzle: puzzles::Puzzle,

    /// List of [`path::Path`]. The number of paths equals to the number of puzzles to print.
    pub paths: Vec<path::Path>,

    /// List of diamonds. The number of objects equals to the number of puzzles to print.
    pub diamonds: Vec<Vec<(usize, usize)>>,

    /// List of maps. The number of objects equals to the number of puzzles to print.
    pub maps: Vec<Vec<usize>>,

    /// Number of puzzles to print.
    pub n_puzzles: usize,

    /// Number of puzzles per page.
    pub n_puzzles_per_page: u32,

    /// Whether to print the solutions. The solutions are printed after the puzzles, on seperate
    /// pages. If solutions must be printed, then the number of pages is doubled.
    pub solutions: bool,
}

mod imp {
    use super::*;
    use std::cell::OnceCell;

    #[derive(Default)]
    pub struct HexkudoPrintJob {
        pub parameters: OnceCell<PrintJobParameters>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for HexkudoPrintJob {
        const NAME: &'static str = "HexkudoPrintJob";
        type Type = super::HexkudoPrintJob;
        type ParentType = gtk::PrintOperation;
    }

    impl ObjectImpl for HexkudoPrintJob {}
    impl PrintOperationImpl for HexkudoPrintJob {
        fn begin_print(&self, _context: &gtk::PrintContext) {
            self.obj().begin_print();
        }
        fn draw_page(&self, context: &gtk::PrintContext, page_nr: i32) {
            self.obj().draw_page(context, page_nr);
        }
    }
    impl PrintOperationPreviewImpl for HexkudoPrintJob {
        fn render_page(&self, _: i32) {}
        fn is_selected(&self, _: i32) -> bool {
            false
        }
        fn end_preview(&self) {}
    }
}

glib::wrapper! {
    pub struct HexkudoPrintJob(ObjectSubclass<imp::HexkudoPrintJob>)
        @extends gtk::PrintOperation,                @implements gtk::PrintOperationPreview;
}

impl HexkudoPrintJob {
    /// Create the print operation object.
    pub fn new(parameters: PrintJobParameters) -> Self {
        let obj: HexkudoPrintJob = glib::Object::builder().build();

        if log_enabled!(Level::Debug) {
            debug!("Creating a print job");
            debug!("         puzzle name={}", parameters.puzzle.name);
            debug!("   puzzle difficulty={}", parameters.puzzle.difficulty);
            debug!("           n_puzzles={}", parameters.n_puzzles);
            debug!("  n_puzzles_per_page={}", parameters.n_puzzles_per_page);
            debug!("           solutions={}", parameters.solutions);
            debug!("               paths=");
            for p in &parameters.paths {
                debug!("  - {:?}", p.get());
            }
        }

        obj.imp()
            .parameters
            .set(parameters)
            .expect("Cannot store the print parameters in the object");
        obj
    }

    /// Initiate the print process.
    pub fn print(&self) {
        let imp: &imp::HexkudoPrintJob = self.imp();
        let p: &PrintJobParameters = imp
            .parameters
            .get()
            .expect("Cannot retrieve the printing parameters");
        let window: &gtk::Window = &p.window;

        match self.run(gtk::PrintOperationAction::PrintDialog, Some(window)) {
            Ok(_) => (),
            Err(e) => {
                let dialog: adw::AlertDialog = adw::AlertDialog::new(
                    Some(&gettext("Error Printing Puzzles")),
                    Some(e.message()),
                );
                dialog.add_response("close", &gettext("Close"));
                dialog.present(Some(window));
            }
        }
    }

    /// Callback for when the printing process is initiated.
    pub fn begin_print(&self) {
        let imp: &imp::HexkudoPrintJob = self.imp();
        let p: &PrintJobParameters = imp
            .parameters
            .get()
            .expect("Cannot retrieve the printing parameters");
        let mut pages: i32 = (p.n_puzzles as f32 / p.n_puzzles_per_page as f32).ceil() as i32;

        if p.solutions {
            pages *= 2;
        }
        self.set_n_pages(pages);
    }

    /// Draw the given page
    pub fn draw_page(&self, context: &gtk::PrintContext, page_nr: i32) {
        let imp: &imp::HexkudoPrintJob = self.imp();
        let p: &PrintJobParameters = imp
            .parameters
            .get()
            .expect("Cannot retrieve the printing parameters");
        let mut draw: draw::Draw = draw::Draw::new(&p.puzzle);

        // Always print with the light color scheme
        draw.set_dark(false);

        let ctx: Context = context.cairo_context();
        let width: f64 = context.width();
        let height: f64 = context.height();
        let margin: f64 = 0.02;

        ctx.set_source_rgba(0.0, 0.0, 0.0, 1.0);
        ctx.set_font_size(12.0);
        let label_height: f64 = ctx
            .font_extents()
            .expect("Cannot retrieve the font size")
            .height();
        let (square_size, n_across, n_down) = self.fit_squares_in_rectangle(
            width,
            height,
            label_height,
            margin * draw.surface_size(),
        );

        let margin_x: f64 = (width - square_size * n_across as f64) / (n_across as f64 + 1.0);
        let margin_y: f64 = (height - square_size * n_down as f64) / (n_down as f64 + 1.0);

        let scaling_factor: f64 = (square_size - label_height) / draw.surface_size();

        if log_enabled!(Level::Debug) {
            debug!("Drawing page {page_nr}:");
            debug!("           width = {width}");
            debug!("          height = {height}");
            debug!("  scaling_factor = {scaling_factor}");
            debug!("     square_size = {square_size}");
            debug!("        n_across = {n_across}");
            debug!("          n_down = {n_down}");
            debug!("    label_height = {label_height}");
            debug!("        margin_x = {margin_x}");
            debug!("        margin_y = {margin_y}");
            debug!("          margin = {margin}");
        }

        // Whether to print the puzzles or the solutions
        let solution: bool;
        let mut puzzle_number: usize;
        if page_nr as usize * p.n_puzzles_per_page as usize >= p.n_puzzles {
            solution = true;
            let page: i32 = page_nr - self.n_pages() / 2;
            puzzle_number = page as usize * p.n_puzzles_per_page as usize;
        } else {
            solution = false;
            puzzle_number = page_nr as usize * p.n_puzzles_per_page as usize;
        }

        for i in 0..p.n_puzzles_per_page {
            if puzzle_number >= p.n_puzzles {
                break;
            }
            let cell_x: u32 = i % n_across;
            let cell_y: u32 = i / n_across;
            let x: f64 = margin_x + cell_x as f64 * (square_size + margin_x);
            let y: f64 = margin_y + cell_y as f64 * (square_size + margin_y) + label_height;
            let text: String = if solution {
                format!(
                    "{} - {} {} {}",
                    puzzle_number + 1,
                    p.puzzle.name_i18n,
                    p.puzzle.difficulty,
                    gettext("solution")
                )
            } else {
                format!(
                    "{} - {} {}",
                    puzzle_number + 1,
                    p.puzzle.name_i18n,
                    p.puzzle.difficulty
                )
            };
            let text_extends: TextExtents =
                ctx.text_extents(&text).expect("Cannot get the text size");

            let path: &path::Path = &p.paths[puzzle_number];
            let map: &Vec<usize> = &p.maps[puzzle_number];

            if log_enabled!(Level::Debug) {
                debug!("Page {page_nr}: drawing puzzle {puzzle_number}");
                debug!("    puzzle number on this page = {i}");
                debug!("                      solution = {solution}");
                debug!("                         label = {text}");
                debug!(
                    "                   label width = {}",
                    text_extends.x_advance()
                );
                debug!("                        cell_x = {cell_x}");
                debug!("                        cell_y = {cell_y}");
                debug!("                             x = {x}");
                debug!("                             y = {y}");
            }

            // Draw the puzzle frame
            draw.puzzle_frame().expect("Cannot draw the puzzle frame");

            // Draw the map and diamonds
            draw.puzzle_maps_and_diamonds(path, map, &p.diamonds[puzzle_number])
                .expect("Cannot draw the hints and the diamonds");

            // Draw the cell numbers. If printing the solution, then display all the cell numbers.
            let m: &Vec<usize> = if solution { path.get() } else { map };
            let number_surface: Surface = draw
                .puzzle_cell_numbers(path, m, draw::ZoomLevel::Medium)
                .expect("Cannot draw the cell numbers");
            let path: Option<Surface> = if solution {
                // Draw the solution path (line) over the puzzle
                Some(draw.path(path).expect("Cannot draw the solution path"))
            } else {
                None
            };

            ctx.move_to(x + square_size / 2.0 - text_extends.x_advance() / 2.0, y);
            let _ = ctx.show_text(&text);

            // Paint the puzzle layers
            let _ = ctx.save();
            ctx.translate(x, y + label_height);
            ctx.scale(scaling_factor, scaling_factor);
            let _ = ctx.set_source_surface(draw.background_surface(), 0.0, 0.0);
            let _ = ctx.paint();
            let _ = ctx.set_source_surface(draw.border_surface(), 0.0, 0.0);
            let _ = ctx.paint();
            let _ = ctx.set_source_surface(number_surface, 0.0, 0.0);
            let _ = ctx.paint();
            if let Some(p) = path {
                let _ = ctx.set_source_surface(p, 0.0, 0.0);
                let _ = ctx.paint();
            }
            let _ = ctx.restore();
            puzzle_number += 1;
        }
    }

    /// Compute the size of each puzzle on the page.
    ///
    /// Return a tuple with the following items:
    ///
    /// - The size of the square allocated to the puzzle
    /// - The number of puzzles to draw on a row
    /// - The number of lines of puzzles
    fn fit_squares_in_rectangle(
        &self,
        width: f64,
        height: f64,
        label_height: f64,
        margin: f64,
    ) -> (f64, u32, u32) {
        let imp: &imp::HexkudoPrintJob = self.imp();
        let p: &PrintJobParameters = imp
            .parameters
            .get()
            .expect("Cannot retrieve the printing parameters");
        let n: u32 = p.n_puzzles_per_page;
        let mut best_square_size: f64 = 0.0;
        let mut across: u32 = 1;
        let mut down: u32 = n;

        let mut n_across: u32 = 1;

        // Start by one puzzle per row, and compute the square size. Then iterate with two puzzles
        // per line, ... At then end, keep the layout that provides the largest square size.
        while n_across <= n {
            let n_down: u32 = n.div_ceil(n_across);
            let across_size: f64 = (width - ((n_across as f64 + 1.0) * margin)) / n_across as f64;
            let down_size: f64 =
                (height - ((n_down as f64 + 1.0) * margin) - n_down as f64 * label_height)
                    / n_down as f64;

            let square_size: f64 = if across_size < down_size {
                across_size
            } else {
                down_size
            };
            if square_size > best_square_size {
                best_square_size = square_size;
                across = n_across;
                down = n_down;
            }

            n_across += 1;
        }

        (best_square_size, across, down)
    }
}
