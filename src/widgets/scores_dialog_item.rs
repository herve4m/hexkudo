/*
scores_dialog_items.rs

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

//! Puzzle list item used in the select puzzle combo box in the score dialog.

use adw::subclass::prelude::*;
use gtk::glib;

pub struct Entry {
    pub name: String,
}

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/herve4m/Hexkudo/ui/scores_dialog_item.ui")]
    pub struct HexkudoScoreItem {
        #[template_child]
        pub name: TemplateChild<gtk::Inscription>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for HexkudoScoreItem {
        const NAME: &'static str = "HexkudoScoreItem";
        type Type = super::HexkudoScoreItem;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            // When inheriting from GtkWidget directly, you have to either override the
            // size_allocate/measure functions of WidgetImpl trait or use a layout
            // manager which provides those functions for your widgets like below.
            klass.set_layout_manager_type::<gtk::BinLayout>();
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for HexkudoScoreItem {
        fn dispose(&self) {
            self.dispose_template();
        }
    }

    impl WidgetImpl for HexkudoScoreItem {}
}

glib::wrapper! {
    pub struct HexkudoScoreItem(ObjectSubclass<imp::HexkudoScoreItem>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for HexkudoScoreItem {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl HexkudoScoreItem {
    pub fn set_entry(&self, entry: &Entry) {
        self.imp().name.set_markup(Some(&entry.name));
    }
}
