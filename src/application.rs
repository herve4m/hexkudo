/*
Based on GNOME Sudoku at https://gitlab.gnome.org/GNOME/gnome-sudoku

application.rs

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

use gettextrs::gettext;
use log::debug;
use std::collections::HashMap;

use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::{Variant, WeakRef};
use gtk::{gio, glib};

use crate::config;
use crate::game::Game;
use crate::generator::puzzles;
use crate::saver::game::SaverGame;
use crate::widgets::preferences_dialog::HexkudoPreferencesDialog;
use crate::widgets::print_dialog::HexkudoPrintDialog;
use crate::widgets::window::HexkudoWindow;

mod imp {
    use super::*;
    use std::cell::{OnceCell, RefCell};
    use std::rc::Rc;

    pub struct HexkudoApplication {
        /// The [`HexkudoWindow`] object.
        pub window: OnceCell<WeakRef<HexkudoWindow>>,

        /// The list of puzzles.
        pub puzzle_list: OnceCell<HashMap<(String, puzzles::Difficulty), puzzles::Puzzle>>,

        /// GSettings object.
        pub settings: gio::Settings,

        /// The [`Game`] object stores the parameters of the currently played game.
        pub game: Rc<RefCell<Game>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for HexkudoApplication {
        const NAME: &'static str = "HexkudoApplication";
        type Type = super::HexkudoApplication;
        type ParentType = adw::Application;

        fn new() -> Self {
            let c: OnceCell<HashMap<(String, puzzles::Difficulty), puzzles::Puzzle>> =
                OnceCell::new();
            c.set(puzzles::puzzle_map())
                .expect("Cannot build the puzzle list");
            Self {
                window: OnceCell::new(),
                puzzle_list: c,
                settings: gio::Settings::new(config::APPLICATION_ID),
                game: Rc::default(),
            }
        }
    }

    impl ApplicationImpl for HexkudoApplication {
        // We connect to the activate callback to create a window when the application
        // has been launched. Additionally, this callback notifies us when the user
        // tries to launch a "second instance" of the application. When they try
        // to do that, we'll just present any existing window.
        fn activate(&self) {
            let application = self.obj();
            let window: HexkudoWindow = application.get_main_window();
            // Ask the window manager/compositor to present the window
            window.present();
        }

        // Entry point for GApplication
        fn startup(&self) {
            self.parent_startup();

            let application = self.obj();

            debug!("Getting the saved game");
            let saver: SaverGame = SaverGame::new(glib::user_data_dir());
            match saver.get_game() {
                Ok(o) => match o {
                    Some(g) => {
                        debug!("Game value = {g:?}");
                        self.game.replace(g);
                    }
                    None => debug!("No saved game"),
                },
                Err(error) => {
                    debug!("Error getting the saved game: {error}");
                    // Delete the file in error for trying to resolve the issue for the next start
                    saver.delete_save();
                }
            }

            application.setup_gactions();

            let window: HexkudoWindow = HexkudoWindow::new(
                &*application,
                &self.settings,
                &self.game,
                self.puzzle_list
                    .get()
                    .expect("Cannot retrieve the puzzle list from the object"),
            );
            self.window
                .set(window.downgrade())
                .expect("Failed to initialize the application window");

            application
                .get_main_window()
                .action_set_enabled("app.new-game", self.game.borrow().started);
            application
                .get_main_window()
                .action_set_enabled("app.new-game-same-puzzle", self.game.borrow().started);
            application
                .get_main_window()
                .action_set_enabled("game-view.print-current", false);
            application
                .get_main_window()
                .action_set_enabled("game-view.zoom-out", false);
            application
                .get_main_window()
                .action_set_enabled("game-view.zoom-in", false);

            application.set_accels_for_action("app.quit", &["<Primary>q"]);
            application.set_accels_for_action("app.new-game", &["<Primary>n"]);
            application.set_accels_for_action("app.preferences", &["<Primary>comma"]);
            application.set_accels_for_action("app.help", &["F1"]);
            application.set_accels_for_action("app.toggle-fullscreen", &["F11", "f"]);
            application.set_accels_for_action("app.back-start", &["<Alt>Left", "<Alt>KP_Left"]);
        }

        // Saving the currently played game (if any) on application shutdown.
        fn shutdown(&self) {
            self.parent_shutdown();

            debug!("Saving the game");
            let saver: SaverGame = SaverGame::new(glib::user_data_dir());
            let game = self.game.borrow();
            if game.started && !game.solved {
                match saver.save_game(&game) {
                    Ok(()) => (),
                    Err(error) => debug!("Error saving the game: {error}"),
                }
            } else {
                saver.delete_save();
            }
        }

        // Command line is processed by clap.

        /*
        fn command_line(&self, args: &gio::ApplicationCommandLine) -> glib::ExitCode {
            fn handle_local_options(&self, options: &glib::VariantDict) -> glib::ExitCode {
                println!("Handling command-line...");
                println!("{:?}", options.lookup_value("version", None));
                match options.lookup_value("version", None) {
                    None => (),
                    Some(_) => {
                        println!(
                            "XXX
        Copyright (C) 2025 Hervé Quatremain
        License GPLv3+: GNU GPL version 3 or later <https://gnu.org/licenses/gpl.html>
        This is free software: you are free to change and redistribute it.
        There is NO WARRANTY, to the extent permitted by law."
                        );
                        return glib::ExitCode::from(0);
                    }
                }
                println!("{:?}", options.lookup_value("debug", None));
                match options.lookup_value("debug", None) {
                    None => (),
                    Some(_) => {
                        println!("debug");
                        return glib::ExitCode::from(0);
                    }
                }
                println!("no match");
                glib::ExitCode::from(-1)
            }
            match option.expect("sdfsfd") {
                None => (),
                Some(_) => {
                    println!(
                        "XXX
        Copyright (C) 2025 Hervé Quatremain
        License GPLv3+: GNU GPL version 3 or later <https://gnu.org/licenses/gpl.html>
        This is free software: you are free to change and redistribute it.
        There is NO WARRANTY, to the extent permitted by law."
                    );
                    glib::ExitCode::from(0);
                }
            }
            match options.lookup("version") {
                Err(_) => {
                    glib::ExitCode::from(1);
                }
                Ok(o) => match o {
                    None => (),
                    Some(_) => {
                        println!(
                            "XXX
        Copyright (C) 2025 Hervé Quatremain
        License GPLv3+: GNU GPL version 3 or later <https://gnu.org/licenses/gpl.html>
        This is free software: you are free to change and redistribute it.
        There is NO WARRANTY, to the extent permitted by law."
                        );
                        glib::ExitCode::from(0);
                    }
                },
            }
            println!("no match");
            glib::ExitCode::from(-1)
        }
        */
    }

    impl ObjectImpl for HexkudoApplication {}
    impl GtkApplicationImpl for HexkudoApplication {}
    impl AdwApplicationImpl for HexkudoApplication {}
}

glib::wrapper! {
    pub struct HexkudoApplication(ObjectSubclass<imp::HexkudoApplication>)
        @extends gio::Application, gtk::Application, adw::Application,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl Default for HexkudoApplication {
    fn default() -> Self {
        Self::new()
    }
}

impl HexkudoApplication {
    /// Create an [`HexkudoApplication`] object.
    pub fn new() -> Self {
        glib::Object::builder()
            .property("application-id", config::APPLICATION_ID)
            .property("resource-base-path", "/io/github/herve4m/Hexkudo") //"/io/github/herve4m/Hexkudo")
            .build()
    }

    fn setup_gactions(&self) {
        let actions = [
            gio::ActionEntryBuilder::new("about")
                .activate(|app: &Self, _, _| app.show_about())
                .build(),
            gio::ActionEntryBuilder::new("preferences")
                .activate(|app: &Self, _, _| app.show_preferences())
                .build(),
            gio::ActionEntryBuilder::new("help")
                .activate(|app: &Self, _, _| app.help())
                .build(),
            gio::ActionEntryBuilder::new("quit")
                .activate(|app: &Self, _, _| app.quit())
                .build(),
            gio::ActionEntryBuilder::new("back-start")
                .activate(move |app: &Self, _, _| app.back_start())
                .build(),
            gio::ActionEntryBuilder::new("select-puzzle")
                .parameter_type(Some(&i32::static_variant_type()))
                .activate(move |app: &Self, _, parameter| {
                    app.select_puzzle(parameter.expect("Cannot get the parameter"));
                })
                .build(),
            gio::ActionEntryBuilder::new("start-game")
                .activate(move |app: &Self, _, _| app.start_game())
                .build(),
            gio::ActionEntryBuilder::new("new-game")
                .activate(move |app: &Self, _, _| app.new_game())
                .build(),
            gio::ActionEntryBuilder::new("new-game-same-puzzle")
                .activate(move |app: &Self, _, _| app.new_game_same_puzzle())
                .build(),
            gio::ActionEntryBuilder::new("scores")
                .activate(move |app: &Self, _, _| app.scores())
                .build(),
            gio::ActionEntryBuilder::new("print-multiple")
                .activate(move |app: &Self, _, _| app.print_multiple())
                .build(),
            gio::ActionEntryBuilder::new("toggle-fullscreen")
                .activate(move |app: &Self, _, _| app.toggle_fullscreen())
                .build(),
        ];

        self.add_action_entries(actions);
    }

    fn get_main_window(&self) -> HexkudoWindow {
        self.imp().window.get().unwrap().clone().upgrade().unwrap()
    }

    fn show_about(&self) {
        let window: gtk::Window = self.active_window().unwrap();
        let about: adw::AboutDialog = adw::AboutDialog::builder()
            .application_name("Hexkudo")
            .application_icon(config::APPLICATION_ID)
            .developer_name("Hervé Quatremain")
            .version(config::VERSION)
            .developers(vec!["Hervé Quatremain"])
            // Translators: Replace "translator-credits" with your name/username, and optionally
            // an email or URL.
            .translator_credits(gettext("translator-credits"))
            .copyright("© 2025 Hervé Quatremain")
            .license_type(gtk::License::Gpl30)
            .issue_url("https://github.com/herve4m/hexkudo/issues")
            .website("https://github.com/herve4m/hexkudo")
            .build();
        about.add_credit_section(
            Some(&gettext("Inspiration")),
            &[
                "GNOME Sudoku https://gitlab.gnome.org/GNOME/gnome-sudoku",
                "Open Sudoku https://gitlab.com/opensudoku/opensudoku",
            ],
        );
        about.present(Some(&window));
    }

    fn show_preferences(&self) {
        let window: gtk::Window = self.active_window().unwrap();
        let settings: &gio::Settings = &self.imp().settings;
        let preferences_window: HexkudoPreferencesDialog = HexkudoPreferencesDialog::new(settings);
        preferences_window.present(Some(&window));
    }

    /*
    Does not work with flatpak.
    See https://gitlab.gnome.org/GNOME/gtk/-/issues/6135

    fn help(&self) {
        let window: gtk::Window = self.active_window().unwrap();
        gtk::UriLauncher::new("help:hexkudo").launch(
            Some(&window),
            Some(&gio::Cancellable::new()),
            |_| (),
        );
    }
    */

    fn help(&self) {
        let window: gtk::Window = self.active_window().unwrap();
        #[expect(
            deprecated,
            reason = "See https://gitlab.gnome.org/GNOME/gtk/-/issues/6135"
        )]
        gtk::show_uri(Some(&window), "help:hexkudo", 0);
    }

    fn select_puzzle(&self, parameter: &Variant) {
        let enum_idx: i32 = parameter
            .get::<i32>()
            .expect("The variant needs to be of type `i32`");
        let enum_val: puzzles::Difficulty =
            puzzles::Difficulty::from_repr(enum_idx).expect("Cannot get the difficulty level");

        let puzzle_list = self
            .imp()
            .puzzle_list
            .get()
            .expect("Cannot retrieve the list of puzzles");
        let puzzles: Vec<&puzzles::Puzzle> = puzzle_list
            .iter()
            .filter(|(key, _)| key.1 == enum_val)
            .map(|(_, p)| p)
            .collect();
        self.get_main_window().go_to_select_puzzle(puzzles);
    }

    fn back_start(&self) {
        debug!("Back to starting page");
        if !self.imp().game.borrow().started {
            self.get_main_window().go_to_start();
        }
    }

    fn start_game(&self) {
        debug!("Switch to the game view");
        self.get_main_window().go_to_game();
    }

    fn new_game(&self) {
        debug!("Start a new game");
        self.imp().game.borrow_mut().clear();
        self.get_main_window().go_to_start();
    }

    fn new_game_same_puzzle(&self) {
        debug!("Start a new game with the previously selected difficulty and puzzle");
        self.get_main_window().play_same_puzzle();
    }

    fn scores(&self) {
        debug!("Display scores");
        self.get_main_window().display_scores();
    }

    fn print_multiple(&self) {
        debug!("Print multiple puzzles");
        let window: gtk::Window = self.active_window().unwrap();
        let settings: &gio::Settings = &self.imp().settings;
        let puzzle_list: &HashMap<(String, puzzles::Difficulty), puzzles::Puzzle> = self
            .imp()
            .puzzle_list
            .get()
            .expect("Cannot retrieve the list of puzzles");
        let print_dialog: HexkudoPrintDialog =
            HexkudoPrintDialog::new(settings, puzzle_list, window.clone());
        print_dialog.present(Some(&window));
    }

    fn toggle_fullscreen(&self) {
        debug!("Toggle fullscreen");
        let window: HexkudoWindow = self.get_main_window();
        if window.is_fullscreen() {
            window.unfullscreen();
        } else {
            window.fullscreen();
        }
    }
}
