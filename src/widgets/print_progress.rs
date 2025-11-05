/*
print_progress.rs

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

//! Progress dialog displayed for when the puzzles to print are building.

use adw::subclass::prelude::*;
use gtk::glib;

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/herve4m/Hexkudo/ui/print_progress.ui")]
    pub struct HexkudoPrintProgress {
        // Template widgets
        #[template_child]
        pub progress: TemplateChild<gtk::ProgressBar>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for HexkudoPrintProgress {
        const NAME: &'static str = "HexkudoPrintProgress";
        type Type = super::HexkudoPrintProgress;
        type ParentType = adw::Dialog;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for HexkudoPrintProgress {}
    impl WidgetImpl for HexkudoPrintProgress {}
    impl AdwDialogImpl for HexkudoPrintProgress {}
}

glib::wrapper! {
    pub struct HexkudoPrintProgress(ObjectSubclass<imp::HexkudoPrintProgress>)
        @extends gtk::Widget, adw::Dialog,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::ShortcutManager;
}

impl Default for HexkudoPrintProgress {
    fn default() -> Self {
        Self::new()
    }
}

impl HexkudoPrintProgress {
    /// Create the object.
    pub fn new() -> Self {
        glib::Object::builder().build()
    }
}
