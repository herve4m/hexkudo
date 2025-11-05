/*
Based on GNOME Sudoku at https://gitlab.gnome.org/GNOME/gnome-sudoku

window.rs

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

//! Hexkudo main window.

use log::debug;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use adw::subclass::prelude::*;
use gtk::prelude::*;
use gtk::{gio, glib};

use super::game_view::HexkudoGameView;
use super::select_puzzle_view::HexkudoSelectPuzzleView;
use super::start_view::HexkudoStartView;
use crate::game::Game;
use crate::generator::puzzles;

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/herve4m/Hexkudo/ui/window.ui")]
    pub struct HexkudoWindow {
        // Template widgets
        #[template_child]
        pub view_stack: TemplateChild<adw::ViewStack>,
        #[template_child]
        pub start_view: TemplateChild<HexkudoStartView>,
        #[template_child]
        pub select_puzzle_view: TemplateChild<HexkudoSelectPuzzleView>,
        #[template_child]
        pub game_view: TemplateChild<HexkudoGameView>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for HexkudoWindow {
        const NAME: &'static str = "HexkudoWindow";
        type Type = super::HexkudoWindow;
        type ParentType = adw::ApplicationWindow;

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

    impl ObjectImpl for HexkudoWindow {}
    impl WidgetImpl for HexkudoWindow {}
    impl WindowImpl for HexkudoWindow {}
    impl ApplicationWindowImpl for HexkudoWindow {}
    impl AdwApplicationWindowImpl for HexkudoWindow {}
}

glib::wrapper! {
    pub struct HexkudoWindow(ObjectSubclass<imp::HexkudoWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::ShortcutManager, gtk::Root, gtk::Native;
}

#[gtk::template_callbacks]
impl HexkudoWindow {
    /// Create the window.
    pub fn new<P: IsA<gtk::Application>>(
        application: &P,
        settings: &gio::Settings,
        game: &Rc<RefCell<Game>>,
        puzzle_list: &HashMap<(String, puzzles::Difficulty), puzzles::Puzzle>,
    ) -> Self {
        debug!("In new()");
        let obj: HexkudoWindow = glib::Object::builder()
            .property("application", application)
            .build();

        settings
            .bind("default-width", &obj, "default-width")
            .build();
        settings
            .bind("default-height", &obj, "default-height")
            .build();
        settings
            .bind("window-is-maximized", &obj, "maximized")
            .build();
        settings
            .bind("window-is-fullscreen", &obj, "fullscreened")
            .build();

        obj.imp().start_view.init(settings);
        obj.imp().select_puzzle_view.init(settings);
        obj.imp().game_view.init(settings, game, puzzle_list);
        if game.borrow().started {
            // obj.action_set_enabled("app.new-game", true);
            obj.continue_game();
        }
        debug!("End new()");
        obj
    }

    pub fn go_to_select_puzzle(&self, puzzles: Vec<&puzzles::Puzzle>) {
        let imp: &imp::HexkudoWindow = self.imp();

        self.action_set_enabled("app.back-start", true);
        self.action_set_enabled("app.new-game", false);
        self.action_set_enabled("app.new-game-same-puzzle", false);
        self.action_set_enabled("game-view.print-current", false);
        self.action_set_enabled("game-view.zoom-out", false);
        self.action_set_enabled("game-view.zoom-in", false);

        imp.select_puzzle_view.get().init_puzzle_list(puzzles);
        imp.view_stack.set_visible_child(&*imp.select_puzzle_view);
    }

    pub fn go_to_start(&self) {
        let imp: &imp::HexkudoWindow = self.imp();

        self.action_set_enabled("app.back-start", true);
        self.action_set_enabled("app.new-game", false);
        self.action_set_enabled("app.new-game-same-puzzle", false);
        self.action_set_enabled("game-view.print-current", false);
        self.action_set_enabled("game-view.zoom-out", false);
        self.action_set_enabled("game-view.zoom-in", false);

        imp.view_stack.set_visible_child(&*imp.start_view);
    }

    pub fn go_to_game(&self) {
        let imp: &imp::HexkudoWindow = self.imp();
        let puzzle: puzzles::Puzzle = imp
            .select_puzzle_view
            .get_selected_puzzle()
            .expect("Cannot retrieve the selected puzzle");

        self.action_set_enabled("app.back-start", false);
        self.action_set_enabled("app.new-game", true);
        self.action_set_enabled("app.new-game-same-puzzle", true);
        self.action_set_enabled("game-view.print-current", true);

        imp.view_stack.set_visible_child(&*imp.game_view);
        imp.game_view.set_puzzle(puzzle);
    }

    pub fn play_same_puzzle(&self) {
        let imp: &imp::HexkudoWindow = self.imp();

        self.action_set_enabled("app.back-start", false);
        self.action_set_enabled("app.new-game", true);
        self.action_set_enabled("app.new-game-same-puzzle", true);
        self.action_set_enabled("game-view.print-current", true);

        imp.view_stack.set_visible_child(&*imp.game_view);
        imp.game_view.play_again();
    }

    pub fn continue_game(&self) {
        let imp: &imp::HexkudoWindow = self.imp();

        self.action_set_enabled("app.back-start", false);
        self.action_set_enabled("app.new-game", true);
        self.action_set_enabled("app.new-game-same-puzzle", true);
        self.action_set_enabled("game-view.print-current", true);

        imp.view_stack.set_visible_child(&*imp.game_view);
        imp.game_view.continue_game();
    }

    pub fn display_scores(&self) {
        self.imp().game_view.display_scores(None);
    }

    #[template_callback]
    fn fullscreened_cb(&self) {
        let imp: &imp::HexkudoWindow = self.imp();
        let is_fullscreen: bool = self.is_fullscreen();

        imp.start_view
            .imp()
            .menu_button
            .set_fullscreen_button(is_fullscreen);
        imp.select_puzzle_view
            .imp()
            .menu_button
            .set_fullscreen_button(is_fullscreen);
        imp.game_view
            .imp()
            .menu_button
            .set_fullscreen_button(is_fullscreen);
    }
}
