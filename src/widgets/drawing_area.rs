/*
drawing_area.rs

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

//! Manage drawings and events in the drawing area.

use log::{Level, debug, log_enabled};
use std::ops::DerefMut;

use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::{Properties, clone};
use gtk::cairo::{ImageSurface, Surface};
use gtk::{gdk, gio, glib};
use std::cell::RefCell;
use std::rc::Rc;

use super::popover_number::HexkudoPopoverNumber;
use super::preferences_dialog::get_rgba;
use super::print_job::{HexkudoPrintJob, PrintJobParameters};
use crate::draw;
use crate::game::{CellStatus, Game};
use crate::generator::diamond_and_map;
use crate::generator::path;
use crate::generator::puzzles;
use crate::generator::vertexes;
use crate::widgets::game_view::HexkudoGameView;

/// Currently dragged cell
#[derive(Debug, Clone, Default)]
pub struct Drag {
    /// X coordinate where the drag operation started.
    pub start_x: f64,

    /// Y coordinate where the drag operation started.
    pub start_y: f64,

    /// List of the cell that have been visited by the drag motion.
    pub cells: Vec<vertexes::CellType>,
}

mod imp {
    use super::*;
    use std::cell::{Cell, OnceCell, RefCell};

    #[derive(Default, Properties, gtk::CompositeTemplate)]
    #[properties(wrapper_type = super::HexkudoDrawingArea)]
    #[template(resource = "/io/github/herve4m/Hexkudo/ui/drawing_area.ui")]
    pub struct HexkudoDrawingArea {
        pub settings: OnceCell<gio::Settings>,
        pub is_dark: Cell<bool>,
        pub cairo_surface: OnceCell<ImageSurface>,
        pub scaling_factor: Cell<f64>,
        pub zoom_level: Cell<draw::ZoomLevel>,
        pub draw: RefCell<draw::Draw>,
        pub game: OnceCell<Rc<RefCell<Game>>>,
        pub drag: RefCell<Drag>,

        // Properties
        #[property(get, set)]
        pub show_warnings: Cell<bool>,
        #[property(get, set)]
        pub show_duplicates: Cell<bool>,
        #[property(get, set)]
        pub draw_path: Cell<bool>,

        // Color properties
        #[property(get, set)]
        pub use_default_color_cell_values: Cell<bool>,
        #[property(get, set)]
        pub use_default_color_cell_wrong: Cell<bool>,
        #[property(get, set)]
        pub use_default_color_bg: Cell<bool>,
        #[property(get, set)]
        pub use_default_color_hint_bg: Cell<bool>,
        #[property(get, set)]
        pub use_default_sel_color_bg: Cell<bool>,
        #[property(get, set)]
        pub use_default_color_borders: Cell<bool>,
        #[property(get, set)]
        pub use_default_color_path: Cell<bool>,
        #[property(get, set)]
        pub sel_thick_border: Cell<bool>,

        // Template widgets
        #[template_child]
        pub popover_number: TemplateChild<HexkudoPopoverNumber>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for HexkudoDrawingArea {
        const NAME: &'static str = "HexkudoDrawingArea";
        type Type = super::HexkudoDrawingArea;
        type ParentType = gtk::DrawingArea;

        fn class_init(klass: &mut Self::Class) {
            debug!("In class_init()");
            klass.bind_template();
            klass.bind_template_instance_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            debug!("In instance_init()");
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for HexkudoDrawingArea {
        fn constructed(&self) {
            self.parent_constructed();

            debug!("In constructed()");
            let style_manager: adw::StyleManager = adw::StyleManager::default();

            self.is_dark.set(style_manager.is_dark());
            style_manager.connect_dark_notify(clone!(
                #[weak(rename_to = mself)]
                self,
                move |style_manager| mself.obj().dark(style_manager)
            ));

            self.obj().set_draw_func(clone!(
                #[weak(rename_to = mself)]
                self,
                move |da, ctx, w, h| mself.obj().draw(da, ctx, w, h)
            ));
        }

        fn dispose(&self) {
            self.dispose_template();
        }
    }
    impl WidgetImpl for HexkudoDrawingArea {}
    impl DrawingAreaImpl for HexkudoDrawingArea {}
}

glib::wrapper! {
    pub struct HexkudoDrawingArea(ObjectSubclass<imp::HexkudoDrawingArea>)
        @extends gtk::Widget, gtk::DrawingArea,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

#[gtk::template_callbacks]
impl HexkudoDrawingArea {
    /// Initialize the object.
    pub fn init(&self, settings: &gio::Settings, game: &Rc<RefCell<Game>>) {
        let imp: &imp::HexkudoDrawingArea = self.imp();

        imp.game
            .set(Rc::clone(game))
            .expect("Cannot store the game data into the object");

        // Gsettings bindings
        settings
            .bind("show-warnings", self, "show-warnings")
            .build();
        settings
            .bind("show-duplicates", self, "show-duplicates")
            .build();
        settings.bind("draw-path", self, "draw-path").build();

        settings
            .bind(
                "use-default-color-cell-values",
                self,
                "use-default-color-cell-values",
            )
            .build();
        settings
            .bind(
                "use-default-color-cell-wrong",
                self,
                "use-default-color-cell-wrong",
            )
            .build();
        settings
            .bind("use-default-color-bg", self, "use-default-color-bg")
            .build();
        settings
            .bind(
                "use-default-color-hint-bg",
                self,
                "use-default-color-hint-bg",
            )
            .build();
        settings
            .bind("use-default-sel-color-bg", self, "use-default-sel-color-bg")
            .build();
        settings
            .bind(
                "use-default-color-borders",
                self,
                "use-default-color-borders",
            )
            .build();
        settings
            .bind("use-default-color-path", self, "use-default-color-path")
            .build();
        settings
            .bind("sel-thick-border", self, "sel-thick-border")
            .build();

        // React to color changes from the Preferences dialog
        settings.connect_changed(
            None,
            clone!(
                #[weak(rename_to = mself)]
                self,
                move |settings, key| {
                    mself.color_changed(settings, key);
                }
            ),
        );

        imp.popover_number.init(settings, game);
        imp.settings
            .set(settings.clone())
            .expect("Cannot store the settings in the object");
    }

    pub fn set_zoom_level(&self, zoom_level: draw::ZoomLevel) {
        self.imp().zoom_level.set(zoom_level);
    }

    pub fn switch_warnings(&self) {
        let imp: &imp::HexkudoDrawingArea = self.imp();

        imp.show_warnings.set(!imp.show_warnings.get());
        self.queue_draw();
    }

    pub fn switch_duplicates(&self) {
        let imp: &imp::HexkudoDrawingArea = self.imp();

        imp.show_duplicates.set(!imp.show_duplicates.get());
        self.queue_draw();
    }

    fn dark(&self, style_manager: &adw::StyleManager) {
        let imp: &imp::HexkudoDrawingArea = self.imp();
        let mut game = imp
            .game
            .get()
            .expect("Cannot retrieve the game data from the object")
            .borrow_mut();
        let is_dark: bool = style_manager.is_dark();

        imp.is_dark.set(is_dark);

        // Redraw the puzzle with the new color set
        self.init_puzzle(&mut game.puzzle);
        self.set_path_from_diamonds_and_map(&game.path, &game.diamonds, &game.map);
    }

    fn draw(&self, _da: &gtk::DrawingArea, ctx: &gtk::cairo::Context, w: i32, h: i32) {
        let imp: &imp::HexkudoDrawingArea = self.imp();
        let draw = imp.draw.borrow();
        let game = imp
            .game
            .get()
            .expect("Cannot retrieve the game data from the object")
            .borrow();

        // Hide the puzzle when in pause by not drawing the puzzle
        if !draw.initialized() || game.paused {
            return;
        }

        let scaling_factor: f64 = if w > h {
            w as f64 / draw.surface_size()
        } else {
            h as f64 / draw.surface_size()
        };

        // Save the scaling factor
        imp.scaling_factor.set(scaling_factor);

        // Draw the puzzle in several layers. From top to bottom:
        //
        // - The cell numbers that the user entered
        // - The mapped numbers
        // - The cell borders and the diamonds
        // - The selected cell background
        // - The cell background

        let _ = ctx.save();
        ctx.scale(scaling_factor, scaling_factor);

        // Paint the background
        let _ = ctx.set_source_surface(draw.background_surface(), 0.0, 0.0);
        let _ = ctx.paint();

        // Paint the selected cell background
        let selection_surface: Surface = draw
            .selected_cell(game.get_selected_cell(), imp.sel_thick_border.get())
            .expect("Cannot create a surface to draw the selected cell background");
        let _ = ctx.set_source_surface(selection_surface, 0.0, 0.0);
        let _ = ctx.paint();

        // Paint the cell borders and the diamonds
        let _ = ctx.set_source_surface(draw.border_surface(), 0.0, 0.0);
        let _ = ctx.paint();

        // Paint the cell numbers that the user entered
        // let selection = game.selection.get_cells();
        let player_input: Vec<CellStatus> = game.get_cells();
        let zoom: draw::ZoomLevel = imp.zoom_level.get();
        let user_surface: Surface = draw
            .user_cell_numbers(
                player_input,
                imp.show_duplicates.get(),
                imp.show_warnings.get(),
                zoom,
            )
            .expect("Cannot create a surface to draw the user cell numbers");
        let _ = ctx.set_source_surface(user_surface, 0.0, 0.0);
        let _ = ctx.paint();

        // Paint the path line over the selected numbers
        if imp.draw_path.get() {
            let path = draw
                .path_from_player_input(&game.player_input)
                .expect("Cannot create a surface to draw the user cell numbers");
            let _ = ctx.set_source_surface(path, 0.0, 0.0);
            let _ = ctx.paint();
        }

        let _ = ctx.restore();
        self.grab_focus();
    }

    pub fn init_puzzle(&self, puzzle: &mut puzzles::Puzzle) {
        let imp: &imp::HexkudoDrawingArea = self.imp();

        // Update the puzzle colors when the player customized the colors in the Preferences dialog
        if let Some(settings) = imp.settings.get() {
            let mut rgba: gdk::RGBA = get_rgba(settings, "color-cell-values");
            puzzle.colors.custom.set_text(
                rgba.red() as f64,
                rgba.green() as f64,
                rgba.blue() as f64,
                rgba.alpha() as f64,
            );
            puzzle
                .colors
                .custom
                .set_custom_text(!settings.boolean("use-default-color-cell-values"));

            rgba = get_rgba(settings, "color-cell-wrong");
            puzzle.colors.custom.set_text_wrong(
                rgba.red() as f64,
                rgba.green() as f64,
                rgba.blue() as f64,
                rgba.alpha() as f64,
            );
            puzzle
                .colors
                .custom
                .set_custom_text_wrong(!settings.boolean("use-default-color-cell-wrong"));

            rgba = get_rgba(settings, "color-cell-bg");
            puzzle.colors.custom.set_bg(
                rgba.red() as f64,
                rgba.green() as f64,
                rgba.blue() as f64,
                rgba.alpha() as f64,
            );
            puzzle
                .colors
                .custom
                .set_custom_bg(!settings.boolean("use-default-color-bg"));

            rgba = get_rgba(settings, "color-cell-hint-bg");
            puzzle.colors.custom.set_bg_map(
                rgba.red() as f64,
                rgba.green() as f64,
                rgba.blue() as f64,
                rgba.alpha() as f64,
            );
            puzzle
                .colors
                .custom
                .set_custom_bg_map(!settings.boolean("use-default-color-hint-bg"));

            rgba = get_rgba(settings, "color-sel-cell-bg");
            puzzle.colors.custom.set_selection(
                rgba.red() as f64,
                rgba.green() as f64,
                rgba.blue() as f64,
                rgba.alpha() as f64,
            );
            puzzle
                .colors
                .custom
                .set_custom_selection(!settings.boolean("use-default-sel-color-bg"));

            rgba = get_rgba(settings, "color-cell-borders");
            puzzle.colors.custom.set_border(
                rgba.red() as f64,
                rgba.green() as f64,
                rgba.blue() as f64,
                rgba.alpha() as f64,
            );
            puzzle
                .colors
                .custom
                .set_custom_border(!settings.boolean("use-default-color-borders"));

            rgba = get_rgba(settings, "color-path");
            puzzle.colors.custom.set_path(
                rgba.red() as f64,
                rgba.green() as f64,
                rgba.blue() as f64,
                rgba.alpha() as f64,
            );
            puzzle
                .colors
                .custom
                .set_custom_path(!settings.boolean("use-default-color-path"));
        }

        let mut draw: draw::Draw = draw::Draw::new(puzzle);

        puzzle.set_dark(imp.is_dark.get());
        draw.set_dark(imp.is_dark.get());
        draw.puzzle_frame().expect("Cannot draw the puzzle frame");
        imp.draw.replace(draw);
        imp.popover_number.set_puzzle(puzzle);
    }

    pub fn set_path_from_diamonds_and_map(
        &self,
        path: &path::Path,
        diamonds: &Vec<(usize, usize)>,
        map: &Vec<usize>,
    ) {
        let imp: &imp::HexkudoDrawingArea = self.imp();
        let draw = imp.draw.borrow();

        if !draw.initialized() {
            return;
        }

        draw.puzzle_maps_and_diamonds(path, map, diamonds)
            .expect("Cannot draw the hints and the diamonds");
        imp.popover_number.set_path(path, map);
        self.queue_draw();
    }

    pub fn set_path(&self, path: &path::Path, diamond_and_map: &diamond_and_map::DiamondAndMap) {
        let (diamonds, map) = diamond_and_map.get_diamond_and_map();

        self.set_path_from_diamonds_and_map(path, &diamonds, &map);
    }

    pub fn print_current(&self) {
        let imp: &imp::HexkudoDrawingArea = self.imp();
        let game = imp
            .game
            .get()
            .expect("Cannot retrieve the game data from the object")
            .borrow();
        let window: gtk::Window = self.root().unwrap().downcast::<gtk::Window>().unwrap();

        let print_job: HexkudoPrintJob = HexkudoPrintJob::new(PrintJobParameters {
            window,
            puzzle: game.puzzle.clone(),
            paths: vec![game.path.clone()],
            maps: vec![game.map.clone()],
            diamonds: vec![game.diamonds.clone()],
            n_puzzles: 1,
            n_puzzles_per_page: 1,
            solutions: true,
        });
        print_job.print();
    }

    fn show_popover(&self, cell_id: usize, cell_x: usize, cell_y: usize) {
        let imp: &imp::HexkudoDrawingArea = self.imp();

        // Compute the rectangle that the popover must point to
        let (s_x, s_y, w, h) =
            imp.draw
                .borrow()
                .inscribed_rectangle(imp.scaling_factor.get(), cell_x, cell_y);
        let r: gdk::Rectangle = gdk::Rectangle::new(s_x as i32, s_y as i32, w as i32, h as i32);

        imp.popover_number.show(r, cell_id);
        self.queue_draw();
    }

    pub fn hide_popover(&self) {
        self.imp().popover_number.hide();
    }

    fn get_game_view(&self) -> HexkudoGameView {
        let mut view_widget: gtk::Widget = self.parent().unwrap();

        loop {
            if view_widget.widget_name() == "game_view" {
                return view_widget.downcast::<HexkudoGameView>().unwrap();
            }
            view_widget = view_widget.parent().unwrap();
        }
    }

    // Callback for the GSettings changed event
    fn color_changed(&self, settings: &gio::Settings, key: &str) {
        let imp: &imp::HexkudoDrawingArea = self.imp();
        let mut game = imp
            .game
            .get()
            .expect("Cannot retrieve the game data from the object")
            .borrow_mut();

        match key {
            "color-cell-values" => {
                let rgba: gdk::RGBA = get_rgba(settings, key);
                game.puzzle.colors.custom.set_text(
                    rgba.red() as f64,
                    rgba.green() as f64,
                    rgba.blue() as f64,
                    rgba.alpha() as f64,
                );
            }
            "color-cell-wrong" => {
                let rgba: gdk::RGBA = get_rgba(settings, key);
                game.puzzle.colors.custom.set_text_wrong(
                    rgba.red() as f64,
                    rgba.green() as f64,
                    rgba.blue() as f64,
                    rgba.alpha() as f64,
                );
            }
            "color-cell-bg" => {
                let rgba: gdk::RGBA = get_rgba(settings, key);
                game.puzzle.colors.custom.set_bg(
                    rgba.red() as f64,
                    rgba.green() as f64,
                    rgba.blue() as f64,
                    rgba.alpha() as f64,
                );
            }
            "color-cell-hint-bg" => {
                let rgba: gdk::RGBA = get_rgba(settings, key);
                game.puzzle.colors.custom.set_bg_map(
                    rgba.red() as f64,
                    rgba.green() as f64,
                    rgba.blue() as f64,
                    rgba.alpha() as f64,
                );
            }
            "color-sel-cell-bg" => {
                let rgba: gdk::RGBA = get_rgba(settings, key);
                game.puzzle.colors.custom.set_selection(
                    rgba.red() as f64,
                    rgba.green() as f64,
                    rgba.blue() as f64,
                    rgba.alpha() as f64,
                );
            }
            "color-cell-borders" => {
                let rgba: gdk::RGBA = get_rgba(settings, key);
                game.puzzle.colors.custom.set_border(
                    rgba.red() as f64,
                    rgba.green() as f64,
                    rgba.blue() as f64,
                    rgba.alpha() as f64,
                );
            }
            "color-path" => {
                let rgba: gdk::RGBA = get_rgba(settings, key);
                game.puzzle.colors.custom.set_path(
                    rgba.red() as f64,
                    rgba.green() as f64,
                    rgba.blue() as f64,
                    rgba.alpha() as f64,
                );
            }
            _ => return,
        }

        let mut draw = imp.draw.borrow_mut();

        draw.replace_puzzle(&game.puzzle);
        draw.puzzle_frame().expect("Cannot draw the puzzle frame");
        draw.puzzle_maps_and_diamonds(&game.path, &game.map, &game.diamonds)
            .expect("Cannot draw the hints and the diamonds");
        self.queue_draw();
    }

    #[template_callback]
    fn refresh_cb(&self) {
        self.queue_draw();
    }

    #[template_callback]
    fn use_default_color_cell_values_cb(&self) {
        let imp: &imp::HexkudoDrawingArea = self.imp();
        let mut game = imp
            .game
            .get()
            .expect("Cannot retrieve the game data from the object")
            .borrow_mut();
        let mut draw = imp.draw.borrow_mut();

        game.puzzle
            .colors
            .custom
            .set_custom_text(!imp.use_default_color_cell_values.get());
        draw.replace_puzzle(&game.puzzle);
        self.queue_draw();
    }

    #[template_callback]
    fn use_default_color_cell_wrong_cb(&self) {
        let imp: &imp::HexkudoDrawingArea = self.imp();
        let mut game = imp
            .game
            .get()
            .expect("Cannot retrieve the game data from the object")
            .borrow_mut();
        let mut draw = imp.draw.borrow_mut();

        game.puzzle
            .colors
            .custom
            .set_custom_text_wrong(!imp.use_default_color_cell_wrong.get());
        draw.replace_puzzle(&game.puzzle);
        self.queue_draw();
    }

    #[template_callback]
    fn use_default_color_bg_cb(&self) {
        let imp: &imp::HexkudoDrawingArea = self.imp();
        let mut game = imp
            .game
            .get()
            .expect("Cannot retrieve the game data from the object")
            .borrow_mut();
        let mut draw = imp.draw.borrow_mut();

        game.puzzle
            .colors
            .custom
            .set_custom_bg(!imp.use_default_color_bg.get());
        draw.replace_puzzle(&game.puzzle);
        draw.puzzle_frame().expect("Cannot draw the puzzle frame");
        draw.puzzle_maps_and_diamonds(&game.path, &game.map, &game.diamonds)
            .expect("Cannot draw the hints and the diamonds");
        self.queue_draw();
    }

    #[template_callback]
    fn use_default_color_hint_bg_cb(&self) {
        let imp: &imp::HexkudoDrawingArea = self.imp();
        let mut game = imp
            .game
            .get()
            .expect("Cannot retrieve the game data from the object")
            .borrow_mut();
        let mut draw = imp.draw.borrow_mut();

        game.puzzle
            .colors
            .custom
            .set_custom_bg_map(!imp.use_default_color_hint_bg.get());
        draw.replace_puzzle(&game.puzzle);
        draw.puzzle_maps_and_diamonds(&game.path, &game.map, &game.diamonds)
            .expect("Cannot draw the hints and the diamonds");
        self.queue_draw();
    }

    #[template_callback]
    fn use_default_sel_color_bg_cb(&self) {
        let imp: &imp::HexkudoDrawingArea = self.imp();
        let mut game = imp
            .game
            .get()
            .expect("Cannot retrieve the game data from the object")
            .borrow_mut();
        let mut draw = imp.draw.borrow_mut();

        game.puzzle
            .colors
            .custom
            .set_custom_selection(!imp.use_default_sel_color_bg.get());
        draw.replace_puzzle(&game.puzzle);
        self.queue_draw();
    }

    #[template_callback]
    fn use_default_color_borders_cb(&self) {
        let imp: &imp::HexkudoDrawingArea = self.imp();
        let mut game = imp
            .game
            .get()
            .expect("Cannot retrieve the game data from the object")
            .borrow_mut();
        let mut draw = imp.draw.borrow_mut();

        game.puzzle
            .colors
            .custom
            .set_custom_border(!imp.use_default_color_borders.get());
        draw.replace_puzzle(&game.puzzle);
        draw.puzzle_frame().expect("Cannot draw the puzzle frame");
        draw.puzzle_maps_and_diamonds(&game.path, &game.map, &game.diamonds)
            .expect("Cannot draw the hints and the diamonds");
        self.queue_draw();
    }

    #[template_callback]
    fn use_default_color_path_cb(&self) {
        let imp: &imp::HexkudoDrawingArea = self.imp();
        let mut game = imp
            .game
            .get()
            .expect("Cannot retrieve the game data from the object")
            .borrow_mut();
        let mut draw = imp.draw.borrow_mut();

        game.puzzle
            .colors
            .custom
            .set_custom_path(!imp.use_default_color_path.get());
        draw.replace_puzzle(&game.puzzle);
        self.queue_draw();
    }

    #[template_callback]
    fn sel_thick_border_cb(&self) {
        self.queue_draw();
    }

    // Callback for drag begin event
    #[template_callback]
    fn drag_begin_cb(&self, x_surface: f64, y_surface: f64, gesture: &gtk::GestureDrag) {
        let imp: &imp::HexkudoDrawingArea = self.imp();
        let draw = imp.draw.borrow();
        let (_x_, _y, cell_type) =
            draw.surface_to_cell_coordinates(imp.scaling_factor.get(), x_surface, y_surface);
        let button: u32 = gesture.current_button();

        // Expect the left or right mouse button for drag motions
        if button != 1 && button != 3 {
            return;
        }

        imp.drag.replace(Drag {
            start_x: x_surface,
            start_y: y_surface,
            cells: vec![cell_type],
        });
        self.hide_popover();
    }

    // Callback for drag update event
    #[template_callback]
    fn drag_update_cb(
        &self,
        offset_x_surface: f64,
        offset_y_surface: f64,
        gesture: &gtk::GestureDrag,
    ) {
        let imp: &imp::HexkudoDrawingArea = self.imp();
        let mut drag = imp.drag.borrow_mut();
        let draw = imp.draw.borrow();
        let mut game = imp
            .game
            .get()
            .expect("Cannot retrieve the game data from the object")
            .borrow_mut();
        let (_x, _y, current_cell) = draw.surface_to_cell_coordinates(
            imp.scaling_factor.get(),
            drag.start_x + offset_x_surface,
            drag.start_y + offset_y_surface,
        );
        let button: u32 = gesture.current_button();

        // Expect the left or right mouse button for drag motions
        if button != 1 && button != 3 {
            return;
        }

        // If the cell has already been visited, then remove all the cells from the selection
        // after that current cell.
        if let Some(i) = drag.cells.iter().position(|c| *c == current_cell) {
            let view: HexkudoGameView = self.get_game_view();

            // Remove the cell values from the puzzle
            for j in i + 1..drag.cells.len() {
                if let vertexes::CellType::Vertex(v) = drag.cells[j]
                    && !game.map.contains(&v)
                {
                    view.remove_cell_value(game.deref_mut(), v);
                }
            }
            // Remove the cell from the list of cells in the drag object
            drag.cells = Vec::from(&drag.cells[0..=i]);
            self.queue_draw();
            return;
        }

        // First time visiting that cell
        if let vertexes::CellType::Vertex(current_cid) = current_cell {
            let previous_cell: &vertexes::CellType = drag
                .cells
                .last()
                .expect("Cannot retrieve the previous visited cell in the drag motion");
            if let vertexes::CellType::Vertex(previous_cid) = previous_cell {
                // If the current and the previous cell are not adjacent, then return
                if !game
                    .puzzle
                    .matrix
                    .vertexes
                    .is_adjacent(current_cid, *previous_cid)
                {
                    return;
                }
                // Get the value of the previous cell, and use this value +- 1 for the
                // current cell
                if let Some(value) = game.player_input.get_value_from_id(*previous_cid) {
                    let next_value: usize = if button == 1 { value + 1 } else { value - 1 };
                    drag.cells.push(current_cell);
                    drop(drag);
                    // Set the value only if the current cell is not a map (hint) cell
                    // and value is not the ending value.
                    if !game.map.contains(&current_cid)
                        && next_value > 0
                        && next_value < game.puzzle.matrix.vertexes.num_vertexes
                    {
                        let view: HexkudoGameView = self.get_game_view();
                        view.set_cell_value(game.deref_mut(), current_cid, next_value);
                    }
                    self.queue_draw();
                }
            }
        }
    }

    // Callback for drag end event
    #[template_callback]
    fn drag_end_cb(
        &self,
        offset_x_surface: f64,
        offset_y_surface: f64,
        gesture: &gtk::GestureDrag,
    ) {
        let imp: &imp::HexkudoDrawingArea = self.imp();
        let drag = imp.drag.borrow();
        let draw = imp.draw.borrow();
        let (x, y, cell_type) = draw.surface_to_cell_coordinates(
            imp.scaling_factor.get(),
            drag.start_x + offset_x_surface,
            drag.start_y + offset_y_surface,
        );
        let button: u32 = gesture.current_button();

        // Expect the left mouse button for mouse release events
        if button != 1 {
            return;
        }

        // The use released the button in the same cell as the starting cell. Show the popover.
        if drag.cells[0] == cell_type {
            match cell_type {
                vertexes::CellType::Vertex(v) => {
                    if imp
                        .game
                        .get()
                        .expect("Cannot retrieve the game data from the object")
                        .borrow()
                        .map
                        .contains(&v)
                    {
                        self.hide_popover();
                    } else {
                        self.show_popover(v, x, y);
                    }
                }
                _ => self.hide_popover(),
            }
        }
    }

    fn move_selection_right(game: &Game, cell_id: Option<usize>) -> Option<usize> {
        let mut cell: usize;
        match cell_id {
            Some(cid) => cell = cid,
            None => return None,
        }
        let adjacent: vertexes::Adjacent = game.puzzle.matrix.vertexes.get_adjacent(cell);

        if let Some(cell_type) = adjacent.e
            && let vertexes::CellType::Vertex(v) = cell_type
        {
            if !game.map.contains(&v) {
                return Some(v);
            }
            return Self::move_selection_right(game, Some(v));
        }
        cell += 1;
        if cell >= game.puzzle.matrix.vertexes.num_vertexes {
            cell = 0;
        }
        if !game.map.contains(&cell) {
            return Some(cell);
        }

        Self::move_selection_right(game, Some(cell))
    }

    fn move_selection_left(game: &Game, cell_id: Option<usize>) -> Option<usize> {
        let mut cell: usize;
        match cell_id {
            Some(cid) => cell = cid,
            None => return None,
        }
        let adjacent: vertexes::Adjacent = game.puzzle.matrix.vertexes.get_adjacent(cell);

        if let Some(cell_type) = adjacent.w
            && let vertexes::CellType::Vertex(v) = cell_type
        {
            if !game.map.contains(&v) {
                return Some(v);
            }

            return Self::move_selection_left(game, Some(v));
        }
        if cell == 0 {
            cell = game.puzzle.matrix.vertexes.num_vertexes;
        }
        cell -= 1;
        if !game.map.contains(&cell) {
            return Some(cell);
        }

        Self::move_selection_left(game, Some(cell))
    }

    fn move_selection_up(game: &Game, cell_id: Option<usize>) -> Option<usize> {
        let cell: usize = cell_id?;
        let adjacent: vertexes::Adjacent = game.puzzle.matrix.vertexes.get_adjacent(cell);
        let new_vertex: Option<usize> = if let Some(cell_type) = adjacent.nw
            && let vertexes::CellType::Vertex(v) = cell_type
        {
            Some(v)
        } else if let Some(cell_type) = adjacent.ne
            && let vertexes::CellType::Vertex(v) = cell_type
        {
            Some(v)
        } else {
            None
        };

        // Move the selection to the upper-left or the upper-right cell if it is a vertex (not a
        // background or a logo), and if it is not a hint (map)
        if let Some(v) = new_vertex {
            if !game.map.contains(&v) {
                return Some(v);
            }
            // If the selection cannot be moved to the previous row, then skip that row by
            // recursively move the selection up from the cell above (the mapped cell)
            return Self::move_selection_up(game, Some(v));
        }

        // Move to the bottom row (wrap to the bottom row)
        if let Some((x, _)) = game.puzzle.matrix.vertexes.get_coordinates(cell) {
            for y in [
                game.puzzle.matrix.vertexes.height - 1,
                game.puzzle.matrix.vertexes.height - 2,
            ] {
                for idx in 0..6 {
                    if let vertexes::CellType::Vertex(v) =
                        game.puzzle.matrix.vertexes.get_cell(x + idx, y)
                        && !game.map.contains(&v)
                    {
                        return Some(v);
                    }
                    if x >= idx
                        && let vertexes::CellType::Vertex(v) =
                            game.puzzle.matrix.vertexes.get_cell(x - idx, y)
                        && !game.map.contains(&v)
                    {
                        return Some(v);
                    }
                }
            }
        }

        None
    }

    fn move_selection_down(game: &Game, cell_id: Option<usize>) -> Option<usize> {
        let cell: usize = cell_id?;
        let adjacent: vertexes::Adjacent = game.puzzle.matrix.vertexes.get_adjacent(cell);
        let new_vertex: Option<usize> = if let Some(cell_type) = adjacent.se
            && let vertexes::CellType::Vertex(v) = cell_type
        {
            Some(v)
        } else if let Some(cell_type) = adjacent.sw
            && let vertexes::CellType::Vertex(v) = cell_type
        {
            Some(v)
        } else {
            None
        };

        // Move the selection to the bottom-right or the bottom-left cell if it is a vertex (not a
        // background or a logo), and if it is not a hint (map)
        if let Some(v) = new_vertex {
            if !game.map.contains(&v) {
                return Some(v);
            }
            // If the selection cannot be moved to the next row, then skip that row by
            // recursively move the selection down from the cell below (the mapped cell)
            return Self::move_selection_down(game, Some(v));
        }

        // Move to the top row (wrap to the top row)
        if let Some((x, _)) = game.puzzle.matrix.vertexes.get_coordinates(cell) {
            for y in 0..2 {
                for idx in 0..6 {
                    if let vertexes::CellType::Vertex(v) =
                        game.puzzle.matrix.vertexes.get_cell(x + idx, y)
                        && !game.map.contains(&v)
                    {
                        return Some(v);
                    }
                    if x >= idx
                        && let vertexes::CellType::Vertex(v) =
                            game.puzzle.matrix.vertexes.get_cell(x - idx, y)
                        && !game.map.contains(&v)
                    {
                        return Some(v);
                    }
                }
            }
        }

        None
    }

    fn number_key(&self, game: &mut Game, number: usize) {
        let selected_cell_id: usize = match game.get_selected_cell() {
            Some(cid) => cid,
            None => return,
        };
        let mut new_value: usize = number;

        if game.is_selected_cell_value_updated()
            && let Some(cell_value) = game.player_input.get_value_from_id(selected_cell_id)
        {
            new_value = cell_value * 10 + number;
            if new_value >= game.puzzle.matrix.vertexes.num_vertexes {
                new_value = number;
            }
        }
        if new_value == 0 {
            return;
        }
        let view: HexkudoGameView = self.get_game_view();
        view.set_cell_value(game, selected_cell_id, new_value);
        game.set_selected_cell_value_updated(true);
        self.queue_draw();
    }

    fn backspace_key(&self, game: &mut Game) {
        let selected_cell_id: usize = match game.get_selected_cell() {
            Some(cid) => cid,
            None => return,
        };
        if let Some(cell_value) = game.player_input.get_value_from_id(selected_cell_id) {
            let new_value: usize = cell_value / 10;
            let view: HexkudoGameView = self.get_game_view();
            if new_value == 0 {
                view.remove_cell_value(game, selected_cell_id);
                game.set_selected_cell_value_updated(false);
            } else {
                view.set_cell_value(game, selected_cell_id, new_value);
                game.set_selected_cell_value_updated(true);
            }
            self.queue_draw();
        }
    }

    // Callback for key events
    #[template_callback]
    fn key_pressed_cb(
        &self,
        keyval: gdk::Key,
        keycode: u32,
        modifier: gdk::ModifierType,
        _controller: &gtk::EventControllerKey,
    ) -> glib::Propagation {
        let imp: &imp::HexkudoDrawingArea = self.imp();
        let mut game = imp
            .game
            .get()
            .expect("Cannot retrieve the game data from the object")
            .borrow_mut();

        // Ctrl shortcuts are managed by the game-view widget
        if !game.started || modifier == gdk::ModifierType::CONTROL_MASK {
            return glib::Propagation::Proceed;
        }

        if log_enabled!(Level::Debug) {
            debug!("Key pressed:");
            debug!("       keyval = {keyval:?}");
            debug!("     modifier = {modifier:?}");
            debug!("      keycode = {keycode}");
        }

        match keyval {
            gdk::Key::Return | gdk::Key::space => {
                let selected_cell_id: usize = match game.get_selected_cell() {
                    Some(cid) => cid,
                    None => return glib::Propagation::Proceed,
                };
                let cell_x: usize;
                let cell_y: usize;
                match game
                    .puzzle
                    .matrix
                    .vertexes
                    .get_coordinates(selected_cell_id)
                {
                    Some(c) => (cell_x, cell_y) = c,
                    None => return glib::Propagation::Proceed,
                }
                drop(game);
                self.show_popover(selected_cell_id, cell_x, cell_y);
                return glib::Propagation::Stop;
            }

            gdk::Key::_0 | gdk::Key::KP_0 => self.number_key(game.deref_mut(), 0),
            gdk::Key::_1 | gdk::Key::KP_1 => self.number_key(game.deref_mut(), 1),
            gdk::Key::_2 | gdk::Key::KP_2 => self.number_key(game.deref_mut(), 2),
            gdk::Key::_3 | gdk::Key::KP_3 => self.number_key(game.deref_mut(), 3),
            gdk::Key::_4 | gdk::Key::KP_4 => self.number_key(game.deref_mut(), 4),
            gdk::Key::_5 | gdk::Key::KP_5 => self.number_key(game.deref_mut(), 5),
            gdk::Key::_6 | gdk::Key::KP_6 => self.number_key(game.deref_mut(), 6),
            gdk::Key::_7 | gdk::Key::KP_7 => self.number_key(game.deref_mut(), 7),
            gdk::Key::_8 | gdk::Key::KP_8 => self.number_key(game.deref_mut(), 8),
            gdk::Key::_9 | gdk::Key::KP_9 => self.number_key(game.deref_mut(), 9),
            gdk::Key::BackSpace => self.backspace_key(game.deref_mut()),

            gdk::Key::ISO_Left_Tab | gdk::Key::Tab => {
                if modifier == gdk::ModifierType::SHIFT_MASK {
                    if let Some(cid) = Self::move_selection_left(&game, game.get_selected_cell()) {
                        game.set_selected_cell(Some(cid));
                        self.hide_popover();
                        self.queue_draw();
                        return glib::Propagation::Stop;
                    }
                } else if let Some(cid) =
                    Self::move_selection_right(&game, game.get_selected_cell())
                {
                    game.set_selected_cell(Some(cid));
                    self.hide_popover();
                    self.queue_draw();
                    return glib::Propagation::Stop;
                }
            }
            gdk::Key::Right | gdk::Key::KP_Right | gdk::Key::d => {
                if let Some(cid) = Self::move_selection_right(&game, game.get_selected_cell()) {
                    game.set_selected_cell(Some(cid));
                    self.hide_popover();
                    self.queue_draw();
                    return glib::Propagation::Stop;
                }
            }
            gdk::Key::Left | gdk::Key::KP_Left | gdk::Key::a => {
                if let Some(cid) = Self::move_selection_left(&game, game.get_selected_cell()) {
                    game.set_selected_cell(Some(cid));
                    self.hide_popover();
                    self.queue_draw();
                    return glib::Propagation::Stop;
                }
            }
            gdk::Key::Up | gdk::Key::KP_Up | gdk::Key::w => {
                if let Some(cid) = Self::move_selection_up(&game, game.get_selected_cell()) {
                    game.set_selected_cell(Some(cid));
                    self.hide_popover();
                    self.queue_draw();
                    return glib::Propagation::Stop;
                }
                // Prevent the up key from leaving the drawing area and reaching the
                // title bar actions
                return glib::Propagation::Stop;
            }
            gdk::Key::Down | gdk::Key::KP_Down | gdk::Key::s => {
                if let Some(cid) = Self::move_selection_down(&game, game.get_selected_cell()) {
                    game.set_selected_cell(Some(cid));
                    self.hide_popover();
                    self.queue_draw();
                    return glib::Propagation::Stop;
                }
            }
            gdk::Key::Delete | gdk::Key::KP_Delete => {
                if let Some(cid) = game.get_selected_cell() {
                    self.get_game_view()
                        .remove_cell_value(game.deref_mut(), cid);
                    game.set_selected_cell_value_updated(false);
                    self.queue_draw();
                }
            }
            gdk::Key::Escape => {
                self.hide_popover();
            }
            _ => (),
        }

        glib::Propagation::Proceed
    }

    #[template_callback]
    fn focus_leave_cb(&self) {
        self.hide_popover();
    }
}
