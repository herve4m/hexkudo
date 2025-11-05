/*
puzzle_list_items.rs

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

//! Puzzle list item in the select puzzle view.

use gettextrs::gettext;
use log::debug;

use adw::subclass::prelude::*;
use gtk::glib;
use gtk::prelude::*;

use crate::generator::puzzles;

mod imp {
    use super::*;
    use std::cell::{Cell, OnceCell};

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/herve4m/Hexkudo/ui/puzzle_list_item.ui")]
    pub struct HexkudoPuzzleListItem {
        // Puzzle object
        pub puzzle: OnceCell<puzzles::Puzzle>,

        // Whether this item represents a random puzzle
        pub random: Cell<bool>,

        // Template widgets
        #[template_child]
        pub check_button: TemplateChild<gtk::CheckButton>,
        #[template_child]
        pub image: TemplateChild<gtk::Picture>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for HexkudoPuzzleListItem {
        const NAME: &'static str = "HexkudoPuzzleListItem";
        type Type = super::HexkudoPuzzleListItem;
        type ParentType = adw::ActionRow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for HexkudoPuzzleListItem {
        fn dispose(&self) {
            self.dispose_template();
        }
    }

    impl WidgetImpl for HexkudoPuzzleListItem {}
    impl ListBoxRowImpl for HexkudoPuzzleListItem {}
    impl PreferencesRowImpl for HexkudoPuzzleListItem {}
    impl ActionRowImpl for HexkudoPuzzleListItem {}
}

glib::wrapper! {
    pub struct HexkudoPuzzleListItem(ObjectSubclass<imp::HexkudoPuzzleListItem>)
        @extends gtk::Widget, gtk::ListBoxRow, adw::PreferencesRow, adw::ActionRow,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Actionable;
}

impl HexkudoPuzzleListItem {
    /// Create a list item object.
    pub fn new(puzzle: &puzzles::Puzzle, random: bool) -> Self {
        let title: String;
        let mut resource_icon: String =
            String::from("/io/github/herve4m/Hexkudo/icons/scalable/actions/");
        if random {
            title = gettext("Random");
            resource_icon += "random.svg";
        } else {
            title = puzzle.name_i18n.clone();
            resource_icon += &puzzle.icon;
        }
        debug!(
            "item name: {}  title: {}  icon: {}",
            puzzle.name, title, resource_icon
        );

        let obj: HexkudoPuzzleListItem = glib::Object::builder().property("title", &title).build();
        obj.imp().image.set_resource(Some(&resource_icon));
        obj.imp()
            .puzzle
            .set(puzzle.clone())
            .expect("Cannot store the puzzle in the object");
        obj.imp().random.set(random);
        obj
    }

    /// Return the name of the puzzle associated with this object.
    pub fn get_puzzle_name(&self) -> &str {
        let imp: &imp::HexkudoPuzzleListItem = self.imp();

        if imp.random.get() {
            "Random"
        } else {
            &imp.puzzle
                .get()
                .expect("Cannot get the puzzle details")
                .name
        }
    }

    /// Mark the current puzzle item as the active (selected) item in the list.
    pub fn set_active(&self) {
        self.imp().check_button.set_active(true);
    }

    /// Whether the current puzzle item is currently selected.
    pub fn is_active(&self) -> bool {
        self.imp().check_button.is_active()
    }
}
