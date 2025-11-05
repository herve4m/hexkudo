/*
Based on GNOME Sudoku at https://gitlab.gnome.org/GNOME/gnome-sudoku

layout_manager.rs

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

//! `GtkLayoutManager` to render the puzzle drawing in the middle of the window.

use adw::subclass::prelude::*;
use gtk::glib;
use gtk::prelude::*;

mod imp {
    use super::*;
    use gtk::Allocation;
    use log::{Level, debug, log_enabled};
    use std::cmp;

    #[derive(Default)]
    pub struct HexkudoLayoutManager {}

    #[glib::object_subclass]
    impl ObjectSubclass for HexkudoLayoutManager {
        const NAME: &'static str = "HexkudoLayoutManager";
        type Type = super::HexkudoLayoutManager;
        type ParentType = gtk::LayoutManager;
    }

    impl ObjectImpl for HexkudoLayoutManager {}
    impl WidgetImpl for HexkudoLayoutManager {}
    impl LayoutManagerImpl for HexkudoLayoutManager {
        /// Compute the size of the square area that the widget expect for rendering the puzzle.
        ///
        /// Return a tuple with the following items:
        ///
        /// - Mininum size
        /// - Natural size
        /// - Mininum baseline (always -1)
        /// - Natural baseline (always -1)
        fn measure(
            &self,
            widget: &gtk::Widget,
            _orientation: gtk::Orientation,
            _for_size: i32,
        ) -> (i32, i32, i32, i32) {
            match widget.first_child() {
                Some(child) => {
                    if !child.is_visible() {
                        debug!("The child is not visible.");
                        return (0, 0, -1, -1);
                    }

                    let (h_min, h_nat, _, _) = child.measure(gtk::Orientation::Horizontal, -1);
                    let (v_min, v_nat, _, _) = child.measure(gtk::Orientation::Vertical, -1);
                    let max_min: i32 = cmp::max(h_min, v_min);
                    let max_nat: i32 = cmp::max(h_nat, v_nat);
                    if log_enabled!(Level::Debug) {
                        debug!("The child is visible:");
                        debug!("        h_min={h_min}");
                        debug!("        v_min={v_min}");
                        debug!("        h_nat={h_nat}");
                        debug!("        v_nat={v_nat}");
                        debug!("    returning=({max_min}, {max_nat}, -1, -1)");
                    }
                    (max_min, max_nat, -1, -1)
                }
                None => (0, 0, -1, -1),
            }
        }

        /// Assign the given size to the given widget.
        fn allocate(&self, widget: &gtk::Widget, width: i32, height: i32, baseline: i32) {
            let child_size: i32;
            let x: i32;
            let y: i32;

            // The widget size (a square) is the lowest between the given height and width.
            // The widget is centered in the given area.
            if width > height {
                child_size = height;
                x = width / 2 - child_size / 2;
                y = 0;
            } else {
                child_size = width;
                x = 0;
                y = height / 2 - child_size / 2;
            }

            if log_enabled!(Level::Debug) {
                debug!("Allocating space for the puzzle drawing:");
                debug!("  given width={width}");
                debug!(" given height={height}");
                debug!("   child_size={child_size}");
                debug!("            x={x}");
                debug!("            y={y}");
            }

            let allocation: Allocation = Allocation::new(x, y, child_size, child_size);
            widget
                .first_child()
                .expect("Cannot retrieve the widget's child")
                .size_allocate(&allocation, baseline);
        }
    }
}

glib::wrapper! {
    pub struct HexkudoLayoutManager(ObjectSubclass<imp::HexkudoLayoutManager>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::LayoutManager;
}

impl Default for HexkudoLayoutManager {
    fn default() -> Self {
        Self::new()
    }
}

impl HexkudoLayoutManager {
    // Create the object.
    pub fn new() -> Self {
        glib::Object::builder().build()
    }
}
