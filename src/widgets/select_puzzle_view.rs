/*
Based on GNOME Sudoku at https://gitlab.gnome.org/GNOME/gnome-sudoku

select_puzzle_view.rs

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

//! Puzzle selection view.

use log::debug;
use rand::Rng;

use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::Properties;
use gtk::{gio, glib};

use super::menu_button::HexkudoMenuButton;
use super::puzzle_list_item::HexkudoPuzzleListItem;
use crate::generator::puzzles;

mod imp {
    use super::*;
    use std::cell::{Cell, RefCell};

    #[derive(Debug, Default, Properties, gtk::CompositeTemplate)]
    #[properties(wrapper_type = super::HexkudoSelectPuzzleView)]
    #[template(resource = "/io/github/herve4m/Hexkudo/ui/select_puzzle_view.ui")]
    pub struct HexkudoSelectPuzzleView {
        // List of the puzzle item widgets
        pub widget_items: RefCell<Vec<HexkudoPuzzleListItem>>,

        // Index in the puzzle list of the puzzle associated with the random entry
        pub rand_id: Cell<Option<usize>>,

        // Properties
        #[property(get, set)]
        pub puzzle: RefCell<String>,

        // Template widgets
        #[template_child]
        pub menu_button: TemplateChild<HexkudoMenuButton>,
        #[template_child]
        pub preference_group: TemplateChild<adw::PreferencesGroup>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for HexkudoSelectPuzzleView {
        const NAME: &'static str = "HexkudoSelectPuzzleView";
        type Type = super::HexkudoSelectPuzzleView;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for HexkudoSelectPuzzleView {}
    impl WidgetImpl for HexkudoSelectPuzzleView {}
    impl BinImpl for HexkudoSelectPuzzleView {}
}

glib::wrapper! {
    pub struct HexkudoSelectPuzzleView(ObjectSubclass<imp::HexkudoSelectPuzzleView>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

#[gtk::template_callbacks]
impl HexkudoSelectPuzzleView {
    /// Initialize the object.
    pub fn init(&self, settings: &gio::Settings) {
        // let imp: &imp::HexkudoSelectPuzzleView = self.imp();

        // Bind the GSettings `puzzle` setting with the puzzle property. This is the name of
        // the puzzle that the user previously selected.
        settings.bind("puzzle", self, "puzzle").build();
    }

    /// Populate the view with the provided list of puzzles.
    pub fn init_puzzle_list(&self, mut puzzles: Vec<&puzzles::Puzzle>) {
        let imp: &imp::HexkudoSelectPuzzleView = self.imp();

        // Clear the list, which might have been set from a previous puzzle selection by the user
        let mut widget_items = imp.widget_items.borrow_mut();
        for w in widget_items.iter() {
            imp.preference_group.get().remove(w);
        }
        widget_items.clear();

        // Select an index in the puzzle list for the random puzzle. To prevent the user from
        // getting the same puzzle when selecting the random item several times, the `rand_id` item
        // is selected at random the first time the puzzle selection view is displayed. After that,
        // `rand_id` is incremented to associate the next puzzle in the list to the random item.
        puzzles.sort_by_key(|&d| &d.name_i18n[..]);
        let rand_id: usize = match imp.rand_id.get() {
            None => {
                let mut rng: rand::prelude::ThreadRng = rand::rng();
                rng.random_range(0..puzzles.len())
            }
            Some(r) => (r + 1) % puzzles.len(),
        };
        imp.rand_id.set(Some(rand_id));

        // Create a widget for each puzzle in the list
        for p in &puzzles {
            let puzzle_widget: HexkudoPuzzleListItem = HexkudoPuzzleListItem::new(p, false);

            // Add the puzzle widget to the Adw.PreferencesGroup widget
            imp.preference_group.get().add(&puzzle_widget);

            // Save the widget in the `widget_items` list
            widget_items.push(puzzle_widget);
        }

        // Append the random puzzle to the widget list
        let puzzle_widget: HexkudoPuzzleListItem =
            HexkudoPuzzleListItem::new(puzzles[rand_id], true);
        imp.preference_group.get().add(&puzzle_widget);
        widget_items.push(puzzle_widget);

        // Name of the puzzle from the `puzzle` GSettings. This puzzle is selected by default in
        // the list.
        let selected_puzzle_name: String = self.puzzle();
        let mut group_defined: bool = false;
        let mut group_widget: Option<&HexkudoPuzzleListItem> = None;
        for w in widget_items.iter() {
            // Group the items as a radio list (only one can be selected at a time)
            if group_defined {
                let g = group_widget
                    .expect("Cannot create the radio buttons")
                    .imp()
                    .check_button
                    .get();
                w.imp().check_button.get().set_group(Some(&g));
            } else {
                group_widget = Some(w);
                // Mark the first item in the list as selected. This might be overwritten when the
                // user previously selected puzzle is encountered.
                w.set_active();
                group_defined = true;
            }
            // User previously selected puzzle
            if w.get_puzzle_name() == selected_puzzle_name {
                w.set_active();
            }
        }
    }

    /// Return the currently selected puzzle
    pub fn get_selected_puzzle(&self) -> Option<puzzles::Puzzle> {
        let imp: &imp::HexkudoSelectPuzzleView = self.imp();

        let widget_items = imp.widget_items.borrow();
        for w in widget_items.iter() {
            if w.is_active() {
                let puzzle: &puzzles::Puzzle =
                    w.imp().puzzle.get().expect("Cannot retrieve the puzzle");
                let puzzle_name: &str = w.get_puzzle_name();
                debug!("Selected puzzle: {puzzle_name}");

                // Update the `puzzle` property and the GSettings `puzzle`
                self.set_puzzle(puzzle_name);
                return Some(puzzle.clone());
            }
        }
        None
    }

    // Callback for the "Start Game" button
    #[template_callback]
    fn start_game_cb(&self, button: &gtk::Button) {
        button
            .activate_action("app.start-game", None)
            .expect("Cannot activate the action to move to the game view");
    }
}
