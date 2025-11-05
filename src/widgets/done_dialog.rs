/*
Based on GNOME Sudoku at https://gitlab.gnome.org/GNOME/gnome-sudoku

done_dialog.rs

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

//! Dialog for when the player successfully completed the puzzle.

use gettextrs::gettext;

use adw::{prelude::*, subclass::prelude::*};
use gtk::glib;

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/herve4m/Hexkudo/ui/done_dialog.ui")]
    pub struct HexkudoDoneDialog {
        // Template widgets
        #[template_child]
        pub highscore_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub highscore_button_content: TemplateChild<adw::ButtonContent>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for HexkudoDoneDialog {
        const NAME: &'static str = "HexkudoDoneDialog";
        type Type = super::HexkudoDoneDialog;
        type ParentType = adw::AlertDialog;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for HexkudoDoneDialog {}
    impl WidgetImpl for HexkudoDoneDialog {}
    impl AdwDialogImpl for HexkudoDoneDialog {}
    impl AdwAlertDialogImpl for HexkudoDoneDialog {}
}

glib::wrapper! {
    pub struct HexkudoDoneDialog(ObjectSubclass<imp::HexkudoDoneDialog>)
        @extends gtk::Widget, adw::Dialog, adw::AlertDialog,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl HexkudoDoneDialog {
    /// Create the dialog.
    pub fn new(cheated: bool, clock_visible: bool, highscore_position: Option<usize>) -> Self {
        let obj: HexkudoDoneDialog = glib::Object::builder().build();
        let imp: &imp::HexkudoDoneDialog = obj.imp();

        let msg: String = if cheated {
            gettext("The puzzle is solved!")
        } else {
            gettext("Well done, you completed the puzzle!")
        };

        obj.set_heading(Some(&msg));

        if clock_visible {
            if let Some(pos) = highscore_position {
                imp.highscore_button_content.set_label(&format!("{pos}"));
            }
        } else {
            imp.highscore_button.set_visible(false);
        }
        obj
    }
}
