/*
Based on GNOME Sudoku at https://gitlab.gnome.org/GNOME/gnome-sudoku

menu_button.rs

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

//! Manage the menu buttons.

use adw::subclass::prelude::*;
use gtk::glib;

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/herve4m/Hexkudo/ui/menu_button.ui")]
    pub struct HexkudoMenuButton {
        // Template widgets
        #[template_child]
        pub menu_fullscreen_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub menu_fullscreen_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub menu_unfullscreen_button: TemplateChild<gtk::Button>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for HexkudoMenuButton {
        const NAME: &'static str = "HexkudoMenuButton";
        type Type = super::HexkudoMenuButton;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for HexkudoMenuButton {}
    impl WidgetImpl for HexkudoMenuButton {}
    impl BinImpl for HexkudoMenuButton {}
}

glib::wrapper! {
    pub struct HexkudoMenuButton(ObjectSubclass<imp::HexkudoMenuButton>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl HexkudoMenuButton {
    pub fn set_fullscreen_button(&self, fullscreen: bool) {
        let imp: &imp::HexkudoMenuButton = self.imp();
        let unfullscreen_button = imp.menu_unfullscreen_button.get();
        let fullscreen_button = imp.menu_fullscreen_button.get();

        if fullscreen {
            imp.menu_fullscreen_stack
                .set_visible_child(&unfullscreen_button);
        } else {
            imp.menu_fullscreen_stack
                .set_visible_child(&fullscreen_button);
        }
    }
}
