/*
Based on GNOME Sudoku at https://gitlab.gnome.org/GNOME/gnome-sudoku

Based on GNOME Sudoku at https://gitlab.gnome.org/GNOME/gnome-sudoku

start_view.rs

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

//! Manage the initial view, which displays the difficulty levels.

use adw::subclass::prelude::*;
use glib::Properties;
use gtk::prelude::*;
use gtk::{gio, glib};

use super::menu_button::HexkudoMenuButton;
use crate::generator::puzzles;

mod imp {
    use super::*;
    use std::cell::Cell;

    #[derive(Debug, Default, Properties, gtk::CompositeTemplate)]
    #[properties(wrapper_type = super::HexkudoStartView)]
    #[template(resource = "/io/github/herve4m/Hexkudo/ui/start_view.ui")]
    pub struct HexkudoStartView {
        // Properties
        #[property(get, set, builder(puzzles::Difficulty::Easy))]
        pub difficulty: Cell<puzzles::Difficulty>,

        // Template widgets
        #[template_child]
        pub menu_button: TemplateChild<HexkudoMenuButton>,
        #[template_child]
        pub easy_check: TemplateChild<gtk::CheckButton>,
        #[template_child]
        pub medium_check: TemplateChild<gtk::CheckButton>,
        #[template_child]
        pub hard_check: TemplateChild<gtk::CheckButton>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for HexkudoStartView {
        const NAME: &'static str = "HexkudoStartView";
        type Type = super::HexkudoStartView;
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
    impl ObjectImpl for HexkudoStartView {}
    impl WidgetImpl for HexkudoStartView {}
    impl BinImpl for HexkudoStartView {}
}

glib::wrapper! {
    pub struct HexkudoStartView(ObjectSubclass<imp::HexkudoStartView>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

#[gtk::template_callbacks]
impl HexkudoStartView {
    /// Initialize the object.
    pub fn init(&self, settings: &gio::Settings) {
        let imp: &imp::HexkudoStartView = self.imp();

        settings.bind("difficulty", self, "difficulty").build();
        match puzzles::Difficulty::from_repr(settings.enum_("difficulty"))
            .expect("Cannot retrieve the default difficulty level")
        {
            puzzles::Difficulty::Easy => imp.easy_check.activate(),
            puzzles::Difficulty::Medium => imp.medium_check.activate(),
            puzzles::Difficulty::Hard => imp.hard_check.activate(),
        };
    }

    #[template_callback]
    fn select_puzzle_cb(&self, button: &gtk::Button) {
        let imp: &imp::HexkudoStartView = self.imp();
        let i: i32;

        if imp.hard_check.is_active() {
            i = puzzles::Difficulty::Hard as i32;
            self.set_difficulty(puzzles::Difficulty::Hard);
        } else if imp.medium_check.is_active() {
            i = puzzles::Difficulty::Medium as i32;
            self.set_difficulty(puzzles::Difficulty::Medium);
        } else {
            i = puzzles::Difficulty::Easy as i32;
            self.set_difficulty(puzzles::Difficulty::Easy);
        }
        // Move to the puzzle selection view
        button
            .activate_action("app.select-puzzle", Some(&i.to_variant()))
            .expect("Cannot activate the action to move to the puzzle selection view");
    }
}
