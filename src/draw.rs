/*
draw.rs

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

//! Draw puzzle components with Cairo.

use log::{Level, debug, log_enabled};
use std::f64::consts::PI;
use strum_macros::FromRepr;

use gtk::cairo::*;
use gtk::gdk;
use gtk::gdk::prelude::TextureExt;

use crate::game::CellStatus;
use crate::generator::path;
use crate::generator::puzzles;
use crate::generator::vertexes;
use crate::player_input::PlayerInput;

const SQRT_3: f64 = 1.732_050_807_568_877_2_f64;
const TWO_DIV_SQRT_3: f64 = 1.154_700_538_379_251_7_f64;

// Size of all the surfaces used in this module. Surfaces are square.
// When applying the surfaces in the DrawingArea object, the surfaces are scaled.
const SURFACE_SIZE: f64 = 1040.0;

/// Zoom level for the cell numbers.
#[derive(Debug, Copy, Clone, PartialEq, Eq, FromRepr, Default, glib::Enum)]
#[repr(i32)]
#[enum_type(name = "ZoomLevel")]
pub enum ZoomLevel {
    Small,
    #[default]
    Medium,
    Large,
}

impl ZoomLevel {
    /// Whether the zoom level is at its smallest.
    pub fn is_fully_zoomed_out(self) -> bool {
        if self == ZoomLevel::Small {
            return true;
        }
        false
    }

    /// Whether the zoom level is at its largest.
    pub fn is_fully_zoomed_in(self) -> bool {
        if self == ZoomLevel::Large {
            return true;
        }
        false
    }

    /// Zoom out and return the new zoom level.
    pub fn zoom_out(self) -> Self {
        match self {
            ZoomLevel::Small | ZoomLevel::Medium => ZoomLevel::Small,
            ZoomLevel::Large => ZoomLevel::Medium,
        }
    }

    /// Zoom in and return the new zoom level.
    pub fn zoom_in(self) -> Self {
        match self {
            ZoomLevel::Small => ZoomLevel::Medium,
            ZoomLevel::Medium | ZoomLevel::Large => ZoomLevel::Large,
        }
    }
}

/// Details of a drawn cell. This is used to quickly identify a cell from its position in the
/// surface.
#[derive(Debug)]
struct DrawCell {
    cell_type: vertexes::CellType,

    // Coordinates in "puzzle" coordinates
    x: usize,
    y: usize,

    // Coordinates in the surface
    surface_x: f64,
    surface_y: f64,
}

/// Draw object that is used to draw the puzzle components.
#[derive(Debug)]
pub struct Draw {
    /// Puzzle's background Cairo surface.
    background_surface: ImageSurface,

    /// Cairo surface for the logos, cell borders, and diamonds.
    border_surface: ImageSurface,

    /// Puzzle object to draw.
    puzzle: puzzles::Puzzle,

    /// Margin size around the puzzle (2% of the surface size).
    margin: f64,

    /// X offset to center the puzzle in the window's width.
    offset_x: f64,

    /// Y offset to center the puzzle in the window's height.
    offset_y: f64,

    /// Scaling factor between the surface size and the puzzle size.
    scaling_factor: f64,

    /// Small surface where the logo is drawn.
    logo_surface: ImageSurface,

    /// Width of the logo.
    logo_width: f64,

    /// Height of the logo.
    logo_height: f64,

    /// Scaling factor to adjust the logo in the cell.
    logo_scaling_factor: f64,

    /// List of cells with their coordinates.
    cells: Vec<DrawCell>,
}

impl Default for Draw {
    fn default() -> Self {
        Self {
            background_surface: ImageSurface::create(Format::ARgb32, 1, 1)
                .expect("Cannot create the background puzzle surface"),
            border_surface: ImageSurface::create(Format::ARgb32, 1, 1)
                .expect("Cannot create the cell border surface"),
            puzzle: puzzles::Puzzle::default(),
            margin: 0.0,
            offset_x: 0.0,
            offset_y: 0.0,
            scaling_factor: 0.0,
            logo_surface: ImageSurface::create(Format::ARgb32, 1, 1)
                .expect("Cannot create the puzzle surface"),
            logo_width: 0.0,
            logo_height: 0.0,
            logo_scaling_factor: 0.0,
            cells: Vec::new(),
        }
    }
}

impl Draw {
    /// Create a [`Draw`] object.
    pub fn new(puzzle: &puzzles::Puzzle) -> Self {
        let background_surface: ImageSurface =
            ImageSurface::create(Format::ARgb32, SURFACE_SIZE as i32, SURFACE_SIZE as i32)
                .expect("Cannot create the background puzzle surface");
        let border_surface: ImageSurface =
            ImageSurface::create(Format::ARgb32, SURFACE_SIZE as i32, SURFACE_SIZE as i32)
                .expect("Cannot create the cell border surface");
        let margin: f64 = SURFACE_SIZE * 0.02;
        let vertexes: &vertexes::Vertexes = &puzzle.matrix.vertexes;
        let puzzle_width: f64 = vertexes.width as f64 + 1.0;
        let puzzle_height: f64 = if vertexes.height.is_multiple_of(2) {
            (vertexes.height as f64 * 3.0 + 1.0) / SQRT_3
        } else {
            (vertexes.height as f64 / 2.0).ceil() * 4.0 / SQRT_3
                + (vertexes.height as f64 / 2.0).floor() * TWO_DIV_SQRT_3
        };
        let scaling_factor: f64;
        let offset_x: f64;
        let offset_y: f64;
        if puzzle_width > puzzle_height {
            scaling_factor = (SURFACE_SIZE - 2.0 * margin) / puzzle_width;
            offset_x = 0.0;
            offset_y = (SURFACE_SIZE - puzzle_height * SURFACE_SIZE / puzzle_width) / 2.0;
        } else {
            scaling_factor = (SURFACE_SIZE - 2.0 * margin) / puzzle_height;
            offset_x = (SURFACE_SIZE - puzzle_width * SURFACE_SIZE / puzzle_height) / 2.0;
            offset_y = 0.0;
        }

        if log_enabled!(Level::Debug) {
            debug!("Parameters:");
            debug!("               margin = {margin}");
            debug!("       vertexes.width = {}", vertexes.width);
            debug!("      vertexes.height = {}", vertexes.height);
            debug!("         puzzle_width = {puzzle_width}");
            debug!("        puzzle_height = {puzzle_height}");
            debug!("       scaling_factor = {scaling_factor}");
            debug!("             offset_x = {offset_x}");
            debug!("             offset_y = {offset_y}");
        }

        // Load the logo icon from the resource and store it in a surface
        let resource_icon: String =
            String::from("/io/github/herve4m/Hexkudo/icons/128x128/actions/") + &puzzle.logo;
        let texture: gdk::Texture = gdk::Texture::from_resource(&resource_icon);
        let texture_downloader: gdk::TextureDownloader = gdk::TextureDownloader::new(&texture);
        let (data, stride) = texture_downloader.download_bytes();
        let logo_width: f64 = texture.width() as f64;
        let logo_height: f64 = texture.height() as f64;
        let scaling_factor_width: f64 = scaling_factor / logo_width;
        let scaling_factor_height: f64 = scaling_factor * SQRT_3 / logo_height;
        let logo_scaling_factor: f64 = if scaling_factor_width > scaling_factor_height {
            scaling_factor_height
        } else {
            scaling_factor_width
        };
        let logo_surface: ImageSurface = ImageSurface::create_for_data(
            data.into_data(),
            Format::ARgb32,
            logo_width as i32,
            logo_height as i32,
            stride as i32,
        )
        .expect("Cannot create cairo surface for the logo");

        if log_enabled!(Level::Debug) {
            debug!("Draw logo:");
            debug!("             logo_width = {logo_width}");
            debug!("            logo_height = {logo_height}");
            debug!("   scaling_factor_width = {scaling_factor_width}");
            debug!("  scaling_factor_height = {scaling_factor_height}");
            debug!("    logo_scaling_factor = {logo_scaling_factor}");
        }

        Self {
            background_surface,
            border_surface,
            puzzle: puzzle.clone(),
            margin,
            offset_x,
            offset_y,
            scaling_factor,
            logo_surface,
            logo_width,
            logo_height,
            logo_scaling_factor,
            cells: Vec::with_capacity(puzzle.matrix.vertexes.num_vertexes),
        }
    }

    /// Set the puzzle object.
    pub fn replace_puzzle(&mut self, puzzle: &puzzles::Puzzle) {
        self.puzzle = puzzle.clone();
    }

    /// Whether the object is initialized or not.
    pub fn initialized(&self) -> bool {
        self.margin > 0.0
    }

    /// Return the puzzle's background Cairo surface.
    pub fn background_surface(&self) -> &ImageSurface {
        &self.background_surface
    }

    /// Return the borders and diamonds Cairo surface.
    pub fn border_surface(&self) -> &ImageSurface {
        &self.border_surface
    }

    /// Return the size of the surface, which is square.
    pub fn surface_size(&self) -> f64 {
        SURFACE_SIZE
    }

    /// Set the color scheme.
    pub fn set_dark(&mut self, is_dark: bool) {
        self.puzzle.set_dark(is_dark);
    }

    /// Draw a puzzle cell.
    ///
    ///              (0, 2/√3)
    ///                 /\
    ///                /  \
    ///               /    \
    ///   (-1, 1/√3) |      | (1, 1/√3)
    ///              |      |
    ///              |      |
    ///              |      |
    ///  (-1, -1/√3)  \    /  (1, -1/√3)
    ///                \  /
    ///                 \/
    ///              (0, -2/√3)
    fn draw_cell_border(&self, ctx: &Context) {
        let pt_top: f64 = self.scaling_factor * TWO_DIV_SQRT_3;
        let pt_mid: f64 = self.scaling_factor / SQRT_3;

        ctx.move_to(0.0, pt_top);
        ctx.line_to(self.scaling_factor, pt_mid);
        ctx.line_to(self.scaling_factor, -pt_mid);
        ctx.line_to(0.0, -pt_top);
        ctx.line_to(-self.scaling_factor, -pt_mid);
        ctx.line_to(-self.scaling_factor, pt_mid);
        ctx.line_to(0.0, pt_top);
    }

    /// Convert cell coordinates to surface coordinates.
    fn cell_to_surface_coordinates(&self, x: usize, y: usize) -> (f64, f64) {
        let x_surface: f64 = (x as f64 + 1.0) * self.scaling_factor + self.margin + self.offset_x;
        let y_surface: f64 = y as f64 * SQRT_3 * self.scaling_factor
            + self.scaling_factor * TWO_DIV_SQRT_3
            + self.margin
            + self.offset_y;

        if log_enabled!(Level::Debug) {
            debug!("  Cell coordinates:");
            debug!("            x = {x}");
            debug!("            y = {y}");
            debug!("    x_surface = {x_surface}");
            debug!("    y_surface = {y_surface}");
        }

        (x_surface, y_surface)
    }

    /// Draw a puzzle cell at the given puzzle coordinate and return the surface coordinates.
    fn draw_cell(&self, x: usize, y: usize, ctx: &Context) -> Result<(f64, f64)> {
        debug!("Draw cell:");
        let (s_x, s_y) = self.cell_to_surface_coordinates(x, y);

        ctx.save()?;
        ctx.translate(s_x, s_y);
        self.draw_cell_border(ctx);
        ctx.restore()?;
        Ok((s_x, s_y))
    }

    /// Draw the start and end cells.
    fn draw_cell_start_end(&self, x: usize, y: usize, ctx: &Context) -> Result<()> {
        debug!("Draw start/end cell:");
        let (s_x, s_y) = self.cell_to_surface_coordinates(x, y);

        ctx.save()?;
        ctx.translate(s_x, s_y);
        ctx.set_line_width(0.1 * self.scaling_factor / 0.8);
        ctx.scale(-0.8, -0.8);
        self.draw_cell_border(ctx);
        ctx.restore()
    }

    /// Draw the logo at the given puzzle coordinate.
    fn draw_logo(&self, x: usize, y: usize, ctx: &Context) -> Result<()> {
        debug!("Draw the logo:");
        let (s_x, s_y) = self.cell_to_surface_coordinates(x, y);

        ctx.save()?;
        ctx.translate(
            s_x - (self.logo_width * self.logo_scaling_factor) / 2.0,
            s_y - (self.logo_height * self.logo_scaling_factor) / 2.0,
        );
        ctx.scale(self.logo_scaling_factor, self.logo_scaling_factor);
        ctx.set_source_surface(&self.logo_surface, 0.0, 0.0)?;
        ctx.paint()?;
        ctx.restore()
    }

    /// Draw a diamond template.
    fn draw_diamond_border(&self, ctx: &Context) {
        let half_width: f64 = self.scaling_factor * 1.0 / 5.0 * SQRT_3;
        let half_height: f64 = half_width / 2.0;

        ctx.move_to(half_width, 0.0);
        ctx.line_to(0.0, -half_height);
        ctx.line_to(-half_width, 0.0);
        ctx.line_to(0.0, half_height);
        ctx.line_to(half_width, 0.0);
    }

    /// Draw the diamond at the left of the given cell coordinates (west).
    fn draw_diamond_w(&self, x: usize, y: usize, ctx: &Context) -> Result<()> {
        debug!("Draw diamond west:");
        let (_x, s_y) = self.cell_to_surface_coordinates(x, y);

        ctx.save()?;
        ctx.translate(
            x as f64 * self.scaling_factor + self.margin + self.offset_x,
            s_y,
        );
        self.draw_diamond_border(ctx);
        ctx.restore()
    }

    /// Draw the diamond at the top left of the given cell coordinates (north-west).
    fn draw_diamond_nw(&self, x: usize, y: usize, ctx: &Context) -> Result<()> {
        debug!("Draw diamond north-west for cell ({x}, {y})");
        ctx.save()?;
        ctx.translate(
            (x as f64 + 0.5) * self.scaling_factor + self.margin + self.offset_x,
            (y as f64 * SQRT_3 - SQRT_3 / 2.0) * self.scaling_factor
                + self.scaling_factor * TWO_DIV_SQRT_3
                + self.margin
                + self.offset_y,
        );
        ctx.rotate(60.0_f64 * PI / 180.0);
        self.draw_diamond_border(ctx);
        ctx.restore()
    }

    /// Draw the diamond at the bottom left of the given cell coordinates (south-west).
    fn draw_diamond_sw(&self, x: usize, y: usize, ctx: &Context) -> Result<()> {
        debug!("Draw diamond south-west for cell ({x}, {y})");
        ctx.save()?;
        ctx.translate(
            (x as f64 + 0.5) * self.scaling_factor + self.margin + self.offset_x,
            (y as f64 * SQRT_3 + SQRT_3 / 2.0) * self.scaling_factor
                + self.scaling_factor * TWO_DIV_SQRT_3
                + self.margin
                + self.offset_y,
        );
        ctx.rotate(-60.0_f64 * PI / 180.0);
        self.draw_diamond_border(ctx);
        ctx.restore()
    }

    /// Draw the cell number by using the provided Cairo context.
    fn draw_cell_number(
        &self,
        number: usize,
        x: usize,
        y: usize,
        ctx: &Context,
        zoom_level: ZoomLevel,
    ) -> Result<()> {
        debug!("Draw cell number:");
        let (s_x, s_y) = self.cell_to_surface_coordinates(x, y);
        let text: String = format!("{number}");

        ctx.save()?;
        match zoom_level {
            ZoomLevel::Large => ctx.set_font_size(1.0 * self.scaling_factor),
            ZoomLevel::Medium => ctx.set_font_size(0.8 * self.scaling_factor),
            ZoomLevel::Small => ctx.set_font_size(0.6 * self.scaling_factor),
        }

        let font_extends: FontExtents = ctx.font_extents()?;
        let text_extends: TextExtents = ctx.text_extents(&text)?;
        let text_width: f64 = text_extends.x_advance();
        let text_height: f64 = font_extends.ascent() + font_extends.descent();

        ctx.move_to(
            s_x - text_width / 2.0,
            s_y + text_height / 2.0 - font_extends.descent(),
        );
        ctx.show_text(&text)?;
        ctx.restore()
    }

    /// Draw the puzzle frame on the puzzle surfaces.
    pub fn puzzle_frame(&mut self) -> Result<()> {
        let vertexes: &vertexes::Vertexes = &self.puzzle.matrix.vertexes;

        // Surface and context where the puzzle is drawn
        let background_puzzle_ctx: Context = Context::new(&self.background_surface)?;
        let border_puzzle_ctx: Context = Context::new(&self.border_surface)?;

        // Clear the puzzle surfaces
        background_puzzle_ctx.set_operator(Operator::Clear);
        background_puzzle_ctx.paint()?;
        background_puzzle_ctx.set_operator(Operator::Over);
        border_puzzle_ctx.set_operator(Operator::Clear);
        border_puzzle_ctx.paint()?;
        border_puzzle_ctx.set_operator(Operator::Over);

        // Colors
        let (bg_cell_r, bg_cell_g, bg_cell_b, bg_cell_a) = self.puzzle.colors.get_bg();
        let (fg_r, fg_g, fg_b, fg_a) = self.puzzle.colors.get_border();

        // Cells color
        background_puzzle_ctx.set_source_rgba(bg_cell_r, bg_cell_g, bg_cell_b, bg_cell_a);

        // Line properties
        border_puzzle_ctx.set_line_width(0.1 * self.scaling_factor);
        border_puzzle_ctx.set_source_rgba(fg_r, fg_g, fg_b, fg_a);
        border_puzzle_ctx.set_line_cap(LineCap::Round);

        self.cells.clear();

        // Draw the cells
        for (x, y, t) in vertexes.iter() {
            match t {
                vertexes::CellType::Vertex(cell_id) => {
                    // Background
                    let (s_x, s_y) = self.draw_cell(x, y, &background_puzzle_ctx)?;
                    background_puzzle_ctx.fill()?;

                    self.cells.push(DrawCell {
                        cell_type: vertexes::CellType::Vertex(cell_id),
                        x,
                        y,
                        surface_x: s_x / self.scaling_factor,
                        surface_y: s_y / self.scaling_factor,
                    });

                    // Border
                    self.draw_cell(x, y, &border_puzzle_ctx)?;
                    border_puzzle_ctx.stroke()?;
                }
                vertexes::CellType::Logo => {
                    // Background
                    let (s_x, s_y) = self.draw_cell(x, y, &background_puzzle_ctx)?;
                    background_puzzle_ctx.fill()?;

                    self.cells.push(DrawCell {
                        cell_type: vertexes::CellType::Logo,
                        x,
                        y,
                        surface_x: s_x / self.scaling_factor,
                        surface_y: s_y / self.scaling_factor,
                    });

                    // Border
                    self.draw_cell(x, y, &border_puzzle_ctx)?;
                    border_puzzle_ctx.stroke()?;

                    // Logo
                    self.draw_logo(x, y, &border_puzzle_ctx)?;
                }
                vertexes::CellType::Background => (),
            }
        }

        if log_enabled!(Level::Debug) {
            border_puzzle_ctx.rectangle(4.0, 4.0, SURFACE_SIZE - 8.0, SURFACE_SIZE - 8.0);
            border_puzzle_ctx.stroke()?;
        }

        Ok(())
    }

    /// Draw the hint cells and diamonds on the Cairo surfaces.
    /// The cell numbers are not drawn at that point.
    pub fn puzzle_maps_and_diamonds(
        &self,
        path: &path::Path,
        map: &Vec<usize>,
        diamonds: &Vec<(usize, usize)>,
    ) -> Result<()> {
        // Surface and context where the map and diamonds are drawn
        let background_puzzle_ctx: Context = Context::new(&self.background_surface)?;
        let border_puzzle_ctx: Context = Context::new(&self.border_surface)?;

        // Colors
        let (bg_map_r, bg_map_g, bg_map_b, bg_map_a) = self.puzzle.colors.get_bg_map();
        let (fg_border_r, fg_border_g, fg_border_b, fg_border_a) = self.puzzle.colors.get_border();
        let (fg_diamond_r, fg_diamond_g, fg_diamond_b, fg_diamond_a) =
            self.puzzle.colors.get_diamond();

        // Map cells background color
        background_puzzle_ctx.set_source_rgba(bg_map_r, bg_map_g, bg_map_b, bg_map_a);

        // Line properties
        border_puzzle_ctx.set_line_width(0.1 * self.scaling_factor);
        border_puzzle_ctx.set_source_rgba(fg_border_r, fg_border_g, fg_border_b, fg_border_a);
        border_puzzle_ctx.set_line_cap(LineCap::Round);

        // Draw the map cells (without the numbers)
        for v in map {
            if let Some(index) = path.vertex_index(*v)
                && let Some((x, y)) = self.puzzle.matrix.vertexes.get_coordinates(*v)
            {
                debug!("Draw background map cell (index = {index}, cell ID = {v})");

                // Background
                self.draw_cell(x, y, &background_puzzle_ctx)?;

                // Borders
                self.draw_cell(x, y, &border_puzzle_ctx)?;

                // Start and end cells
                if index == 0 || index == path.len() - 1 {
                    self.draw_cell_start_end(x, y, &border_puzzle_ctx)?;
                }
            }
        }
        background_puzzle_ctx.fill()?;
        border_puzzle_ctx.stroke()?;

        // Draw the diamonds
        border_puzzle_ctx.set_source_rgba(fg_diamond_r, fg_diamond_g, fg_diamond_b, fg_diamond_a);
        for (v1, v2) in diamonds {
            // let (v1, v2) = d;
            if let Some((x1, y1)) = self.puzzle.matrix.vertexes.get_coordinates(*v1)
                && let Some((x2, y2)) = self.puzzle.matrix.vertexes.get_coordinates(*v2)
            {
                if x2 < x1 {
                    if y2 == y1 {
                        self.draw_diamond_w(x1, y1, &border_puzzle_ctx)?;
                    } else if y2 < y1 {
                        self.draw_diamond_nw(x1, y1, &border_puzzle_ctx)?;
                    } else {
                        self.draw_diamond_sw(x1, y1, &border_puzzle_ctx)?;
                    }
                } else if y2 == y1 {
                    self.draw_diamond_w(x2, y2, &border_puzzle_ctx)?;
                } else if y2 < y1 {
                    self.draw_diamond_sw(x2, y2, &border_puzzle_ctx)?;
                } else {
                    self.draw_diamond_nw(x2, y2, &border_puzzle_ctx)?;
                }
            }
        }
        border_puzzle_ctx.fill()?;

        Ok(())
    }

    /// Draw the numbers of the given map cells on a Cairo surface that is returned.
    pub fn puzzle_cell_numbers(
        &self,
        path: &path::Path,
        map: &Vec<usize>,
        zoom_level: ZoomLevel,
    ) -> Result<Surface> {
        // Surface and context where the numbers are drawn
        let number_surface: ImageSurface =
            ImageSurface::create(Format::ARgb32, SURFACE_SIZE as i32, SURFACE_SIZE as i32)?;
        let number_ctx: Context = Context::new(number_surface)?;
        let (fg_number_r, fg_number_g, fg_number_b, fg_number_a) = self.puzzle.colors.get_text();
        number_ctx.set_source_rgba(fg_number_r, fg_number_g, fg_number_b, fg_number_a);

        for v in map {
            if let Some(index) = path.vertex_index(*v) {
                let (x, y) = self
                    .puzzle
                    .matrix
                    .vertexes
                    .get_coordinates(*v)
                    .expect("Cannot retrieve the cell coordinates 1");

                debug!(
                    "Draw map cell number (index = {}, number = {}, cell ID = {})",
                    index,
                    index + 1,
                    v
                );

                self.draw_cell_number(index + 1, x, y, &number_ctx, zoom_level)?;
            }
        }
        Ok(number_ctx.target())
    }

    /// Draw the user cell values on a Cairo surface that is returned.
    pub fn user_cell_numbers(
        &self,
        cells: Vec<CellStatus>,
        show_duplicate: bool,
        show_errors: bool,
        zoom_level: ZoomLevel,
    ) -> Result<Surface> {
        // Surface and context where the numbers are drawn
        let number_surface: ImageSurface =
            ImageSurface::create(Format::ARgb32, SURFACE_SIZE as i32, SURFACE_SIZE as i32)?;
        let number_ctx: Context = Context::new(number_surface)?;
        let (fg_number_r, fg_number_g, fg_number_b, fg_number_a) = self.puzzle.colors.get_text();
        let (fg_wrong_r, fg_wrong_g, fg_wrong_b, fg_wrong_a) = self.puzzle.colors.get_text_wrong();

        for cell in cells {
            let (x, y) = self
                .puzzle
                .matrix
                .vertexes
                .get_coordinates(cell.cell_id)
                .expect("Cannot retrieve the cell coordinates 2");

            debug!(
                "Draw user cell number (number = {}, cell ID = {}, x = {}, y = {}, duplicate = {} error = {})",
                cell.cell_value, cell.cell_id, x, y, cell.duplicated, cell.error
            );

            if (show_duplicate && cell.duplicated) || (show_errors && cell.error) {
                number_ctx.set_source_rgba(fg_wrong_r, fg_wrong_g, fg_wrong_b, fg_wrong_a);
            } else {
                number_ctx.set_source_rgba(fg_number_r, fg_number_g, fg_number_b, fg_number_a);
            }
            self.draw_cell_number(cell.cell_value, x, y, &number_ctx, zoom_level)?;
        }

        Ok(number_ctx.target())
    }

    /// Draw the selected cell on a Cairo surface that is returned.
    pub fn selected_cell(&self, selected_cell: Option<usize>, thick: bool) -> Result<Surface> {
        // Surface and context where the selected cell is drawn
        let surface: ImageSurface =
            ImageSurface::create(Format::ARgb32, SURFACE_SIZE as i32, SURFACE_SIZE as i32)?;
        let ctx: Context = Context::new(surface)?;

        // Draw the selected cell
        if let Some(cell_id) = selected_cell {
            let (sel_r, sel_g, sel_b, sel_a) = self.puzzle.colors.get_selection();
            ctx.set_source_rgba(sel_r, sel_g, sel_b, sel_a);
            let (x, y) = self
                .puzzle
                .matrix
                .vertexes
                .get_coordinates(cell_id)
                .expect("Cannot retrieve the cell coordinates 3");

            self.draw_cell(x, y, &ctx)?;
            ctx.fill()?;

            if thick {
                let (fg_r, fg_g, fg_b, fg_a) = self.puzzle.colors.get_border();
                ctx.set_source_rgba(fg_r, fg_g, fg_b, fg_a);
                ctx.set_line_width(0.25 * self.scaling_factor);
                ctx.set_line_cap(LineCap::Round);
                self.draw_cell(x, y, &ctx)?;
                ctx.stroke()?;
            }
        }

        Ok(ctx.target())
    }

    /// Draw a line over the path to show the solution on a Cairo surface that is returned.
    pub fn path(&self, path: &path::Path) -> Result<Surface> {
        // Surface and context where the path line is drawn
        let path_surface: ImageSurface =
            ImageSurface::create(Format::ARgb32, SURFACE_SIZE as i32, SURFACE_SIZE as i32)?;
        let path_ctx: Context = Context::new(path_surface)?;
        let (path_r, path_g, path_b, path_a) = self.puzzle.colors.get_path();

        path_ctx.set_source_rgba(path_r, path_g, path_b, path_a);
        path_ctx.set_line_width(0.2 * self.scaling_factor);
        path_ctx.set_line_cap(LineCap::Round);
        path_ctx.set_line_join(LineJoin::Round);

        let mut start: bool = true;
        for v in path.get() {
            let (x, y) = self
                .puzzle
                .matrix
                .vertexes
                .get_coordinates(*v)
                .expect("Cannot retrieve the cell coordinates 4");
            let (s_x, s_y) = self.cell_to_surface_coordinates(x, y);

            if start {
                path_ctx.move_to(s_x, s_y);
                start = false;
            } else {
                path_ctx.line_to(s_x, s_y);
            }
        }
        path_ctx.stroke()?;
        Ok(path_ctx.target())
    }

    /// Draw a line over the adjacent cells that have consecutive values.
    /// The `player_input` parameter is a [`HashMap`]. Keys are cell values, and values are
    /// lists of cell IDs that have this value (the user might have wrongly set several cells with
    /// the same value)
    pub fn path_from_player_input(&self, player_input: &PlayerInput) -> Result<Surface> {
        // Surface and context where the path line is drawn
        let path_surface: ImageSurface =
            ImageSurface::create(Format::ARgb32, SURFACE_SIZE as i32, SURFACE_SIZE as i32)?;
        let path_ctx: Context = Context::new(path_surface)?;
        let (path_r, path_g, path_b, path_a) = self.puzzle.colors.get_path();

        path_ctx.set_source_rgba(path_r, path_g, path_b, path_a);
        path_ctx.set_line_width(0.2 * self.scaling_factor);
        path_ctx.set_line_cap(LineCap::Round);
        path_ctx.set_line_join(LineJoin::Round);

        // Loop over the values
        for i in 1..self.puzzle.matrix.vertexes.num_vertexes {
            // Get the cell ID from the value
            let cell_id_1: usize = match player_input.get_id_from_value(i) {
                Some(cid) => cid,
                None => continue,
            };

            // Get the cell ID for the next value
            let j: usize = i + 1;
            let cell_id_2: usize = match player_input.get_id_from_value(j) {
                Some(cid) => cid,
                None => continue,
            };

            // Verify that the two cells are adjacent
            if !self
                .puzzle
                .matrix
                .vertexes
                .is_adjacent(cell_id_1, cell_id_2)
            {
                continue;
            }
            let (x1, y1) = self
                .puzzle
                .matrix
                .vertexes
                .get_coordinates(cell_id_1)
                .expect("Cannot retrieve the cell coordinates 5");
            let (s_x1, s_y1) = self.cell_to_surface_coordinates(x1, y1);
            let (x2, y2) = self
                .puzzle
                .matrix
                .vertexes
                .get_coordinates(cell_id_2)
                .expect("Cannot retrieve the cell coordinates 6");
            let (s_x2, s_y2) = self.cell_to_surface_coordinates(x2, y2);
            path_ctx.move_to(s_x1, s_y1);
            path_ctx.line_to(s_x2, s_y2);
        }
        path_ctx.stroke()?;

        Ok(path_ctx.target())
    }

    /// Return the coordinates of the cell that matches the given surface coordinates.
    pub fn surface_to_cell_coordinates(
        &self,
        scaling_factor: f64,
        x_surface: f64,
        y_surface: f64,
    ) -> (usize, usize, vertexes::CellType) {
        let surface_x: f64 = x_surface / scaling_factor / self.scaling_factor;
        let surface_y: f64 = y_surface / scaling_factor / self.scaling_factor;

        debug!("Finding clicked cell: surface coordinates ({surface_x}, {surface_y}):");

        for cell in &self.cells {
            let mut dist_x: f64 = (cell.surface_x - surface_x).abs();
            let mut dist_y: f64 = (cell.surface_y - surface_y).abs();

            // Quick checks
            if dist_x > 1.0 || dist_y > TWO_DIV_SQRT_3 {
                continue;
            }
            // The point is in the hexagon incircle
            if dist_x * dist_x + dist_y * dist_y <= 1.0 {
                if log_enabled!(Level::Debug) {
                    match cell.cell_type {
                        vertexes::CellType::Background => debug!(
                            "  Click in background ({}, {}) (quick check)",
                            cell.x, cell.y
                        ),
                        vertexes::CellType::Logo => {
                            debug!("  Click in logo ({}, {}) (quick check)", cell.x, cell.y);
                        }
                        vertexes::CellType::Vertex(v) => debug!(
                            "  Click in vertex {} ({}, {}) (quick check)",
                            v, cell.x, cell.y
                        ),
                    }
                }
                return (cell.x, cell.y, cell.cell_type);
            }

            // For the few cases where the user clicks between the hexagon circumscribing circle
            // and the incircle
            dist_x = (cell.surface_x - surface_x) / TWO_DIV_SQRT_3;
            dist_y = (cell.surface_y - surface_y) / TWO_DIV_SQRT_3;

            if SQRT_3 * dist_y + SQRT_3 > dist_x
                && SQRT_3 * dist_y - SQRT_3 < dist_x
                && -SQRT_3 * dist_y + SQRT_3 > dist_x
                && -SQRT_3 * dist_y - SQRT_3 < dist_x
            {
                if log_enabled!(Level::Debug) {
                    match cell.cell_type {
                        vertexes::CellType::Background => {
                            debug!("  Click in background ({}, {})", cell.x, cell.y);
                        }
                        vertexes::CellType::Logo => {
                            debug!("  Click in logo ({}, {})", cell.x, cell.y);
                        }
                        vertexes::CellType::Vertex(v) => {
                            debug!("  Click in vertex {} ({}, {})", v, cell.x, cell.y);
                        }
                    }
                }
                return (cell.x, cell.y, cell.cell_type);
            }
        }
        debug!("  No cell at the surface coordinates");

        (0, 0, vertexes::CellType::Background)
    }

    /// Return the rectangle coordinates and size inside the cell in surface coordinates.
    pub fn inscribed_rectangle(
        &self,
        scaling_factor: f64,
        cell_x: usize,
        cell_y: usize,
    ) -> (f64, f64, f64, f64) {
        let (s_x, s_y) = self.cell_to_surface_coordinates(cell_x, cell_y);
        let rect_width: f64 = self.scaling_factor * scaling_factor;
        let rect_height: f64 = self.scaling_factor * scaling_factor * 3.0 / SQRT_3;
        (
            s_x * scaling_factor - rect_width / 2.0,
            s_y * scaling_factor - rect_height / 2.0,
            rect_width,
            rect_height,
        )
    }
}
