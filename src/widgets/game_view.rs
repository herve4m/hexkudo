/*
Based on GNOME Sudoku at https://gitlab.gnome.org/GNOME/gnome-sudoku

game_view.rs

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

//! Manage the game view

use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::DerefMut;
use std::rc::Rc;
use std::time::Duration;

use formatx::formatx;
use gettextrs::gettext;
use log::debug;

use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::{Properties, clone};
use gtk::prelude::WidgetExt;
use gtk::{gdk, gio, glib, pango};

use super::drawing_area::HexkudoDrawingArea;
use super::layout_manager::HexkudoLayoutManager;
use super::menu_button::HexkudoMenuButton;
use crate::draw;
use crate::game::Game;
use crate::generator::diamond_and_map;
use crate::generator::diamonds;
use crate::generator::path;
use crate::generator::puzzles::{self, Difficulty};
use crate::generator::random_path;
use crate::highscores::HighScores;
use crate::saver::highscores::SaverHighScores;
use crate::widgets::done_dialog::HexkudoDoneDialog;
use crate::widgets::scores_dialog::HexkudoScoresDialog;

mod imp {
    use super::*;
    use std::cell::{Cell, OnceCell, RefCell};

    #[derive(Debug, Default, Properties, gtk::CompositeTemplate)]
    #[properties(wrapper_type = super::HexkudoGameView)]
    #[template(resource = "/io/github/herve4m/Hexkudo/ui/game_view.ui")]
    pub struct HexkudoGameView {
        pub style_css_provider: OnceCell<gtk::CssProvider>,
        pub game: OnceCell<Rc<RefCell<Game>>>,
        pub puzzle_list: OnceCell<HashMap<(String, Difficulty), puzzles::Puzzle>>,

        // Properties
        #[property(get, set, builder(draw::ZoomLevel::Medium))]
        pub zoom_level: Cell<draw::ZoomLevel>,
        #[property(get, set)]
        pub show_puzzle_bg: Cell<bool>,

        // Template widgets
        #[template_child]
        pub window_title: TemplateChild<adw::WindowTitle>,
        #[template_child]
        pub menu_button: TemplateChild<HexkudoMenuButton>,
        #[template_child]
        pub draw_bin: TemplateChild<adw::Bin>,
        #[template_child]
        pub toast_overlay: TemplateChild<adw::ToastOverlay>,
        #[template_child]
        pub overlay: TemplateChild<gtk::Overlay>,
        #[template_child]
        pub drawing_area: TemplateChild<HexkudoDrawingArea>,
        #[template_child]
        pub spinner: TemplateChild<adw::Spinner>,
        #[template_child]
        pub undo_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub redo_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub play_pause_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub pause_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub play_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub error_box: TemplateChild<gtk::Box>,
        #[template_child]
        pub error_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub clock_box: TemplateChild<gtk::Box>,
        #[template_child]
        pub clock_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub paused_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub resume_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub box_paused: TemplateChild<gtk::Box>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for HexkudoGameView {
        const NAME: &'static str = "HexkudoGameView";
        type Type = super::HexkudoGameView;
        type ParentType = adw::Bin;

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
    impl ObjectImpl for HexkudoGameView {
        fn constructed(&self) {
            self.parent_constructed();

            debug!("In constructed()");
            self.obj().setup_gactions();
            self.style_css_provider
                .set(gtk::CssProvider::new())
                .expect("Cannot store the CSS provider in the object");
            self.draw_bin
                .set_layout_manager(Some(HexkudoLayoutManager::new()));
        }
    }

    impl WidgetImpl for HexkudoGameView {}
    impl BinImpl for HexkudoGameView {}
}

glib::wrapper! {
    pub struct HexkudoGameView(ObjectSubclass<imp::HexkudoGameView>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

#[gtk::template_callbacks]
impl HexkudoGameView {
    /// Initialize the object.
    pub fn init(
        &self,
        settings: &gio::Settings,
        game: &Rc<RefCell<Game>>,
        puzzle_list: &HashMap<(String, puzzles::Difficulty), puzzles::Puzzle>,
    ) {
        let imp: &imp::HexkudoGameView = self.imp();

        // GSettings bindings
        settings.bind("zoom-level", self, "zoom-level").build();
        settings
            .bind("show-timer", &*imp.play_pause_stack, "visible")
            .build();
        settings
            .bind("show-errors", &*imp.error_box, "visible")
            .build();
        settings
            .bind("show-timer", &*imp.clock_box, "visible")
            .build();
        settings
            .bind("show-puzzle-bg", self, "show-puzzle-bg")
            .build();

        imp.drawing_area.init(settings, game);
        imp.drawing_area.set_zoom_level(
            draw::ZoomLevel::from_repr(settings.enum_("zoom-level"))
                .expect("Cannot get the current zoom level"),
        );
        imp.game
            .set(Rc::clone(game))
            .expect("Cannot store the game data into the object");
        imp.puzzle_list
            .set(puzzle_list.clone())
            .expect("Cannot store the puzzle list into the object");

        // Manage the timer widget
        glib::timeout_add_local(
            Duration::new(1, 0),
            clone!(
                #[weak]
                imp,
                #[upgrade_or]
                glib::ControlFlow::Break,
                move || {
                    let game = imp
                        .game
                        .get()
                        .expect("Cannot retrieve the game data from the object")
                        .borrow();
                    if imp.clock_box.is_visible() && !game.paused && !game.solved {
                        let (h, m, s) = game.get_duration_hms();
                        Self::update_clock_widget(&imp, h, m, s);
                    }
                    glib::ControlFlow::Continue
                }
            ),
        );
    }

    fn update_clock_widget(imp: &imp::HexkudoGameView, hour: u64, minute: u64, second: u64) {
        let time_str: String = if hour > 0 {
            format!("{hour:02}:{minute:02}:{second:02}")
        } else {
            format!("{minute:02}:{second:02}")
        };
        imp.clock_label.set_text(&time_str);
    }

    fn update_error_widget(&self, errors: usize) {
        self.imp().error_label.set_text(&format!("{errors}"));
    }

    fn setup_gactions(&self) {
        let group = gio::SimpleActionGroup::new();

        let print_current_action = gio::SimpleAction::new("print-current", None);
        print_current_action.connect_activate(clone!(
            #[weak(rename_to = mself)]
            self,
            move |_, _| mself.print_current_action()
        ));
        group.add_action(&print_current_action);

        let zoom_out_action = gio::SimpleAction::new("zoom-out", None);
        zoom_out_action.connect_activate(clone!(
            #[weak(rename_to = mself)]
            self,
            move |_, _| mself.zoom_out_action()
        ));
        group.add_action(&zoom_out_action);

        let zoom_in_action = gio::SimpleAction::new("zoom-in", None);
        zoom_in_action.connect_activate(clone!(
            #[weak(rename_to = mself)]
            self,
            move |_, _| mself.zoom_in_action()
        ));
        group.add_action(&zoom_in_action);

        let undo_action = gio::SimpleAction::new("undo", None);
        undo_action.connect_activate(clone!(
            #[weak(rename_to = mself)]
            self,
            move |_, _| mself.undo_action()
        ));
        group.add_action(&undo_action);

        let redo_action = gio::SimpleAction::new("redo", None);
        redo_action.connect_activate(clone!(
            #[weak(rename_to = mself)]
            self,
            move |_, _| mself.redo_action()
        ));
        group.add_action(&redo_action);

        let reset_puzzle_action = gio::SimpleAction::new("reset-puzzle", None);
        reset_puzzle_action.connect_activate(clone!(
            #[weak(rename_to = mself)]
            self,
            move |_, _| mself.reset_puzzle_action()
        ));
        group.add_action(&reset_puzzle_action);

        let set_checkpoint = gio::SimpleAction::new("set-checkpoint", None);
        set_checkpoint.connect_activate(clone!(
            #[weak(rename_to = mself)]
            self,
            move |_, _| mself.set_checkpoint_action()
        ));
        group.add_action(&set_checkpoint);

        let undo_checkpoint = gio::SimpleAction::new("undo-checkpoint", None);
        undo_checkpoint.connect_activate(clone!(
            #[weak(rename_to = mself)]
            self,
            move |_, _| mself.undo_checkpoint_action()
        ));
        group.add_action(&undo_checkpoint);

        let solve_cell = gio::SimpleAction::new("solve-current-cell", None);
        solve_cell.connect_activate(clone!(
            #[weak(rename_to = mself)]
            self,
            move |_, _| mself.solve_current_cell_action()
        ));
        group.add_action(&solve_cell);

        let solve_puzzle = gio::SimpleAction::new("solve-puzzle", None);
        solve_puzzle.connect_activate(clone!(
            #[weak(rename_to = mself)]
            self,
            move |_, _| mself.solve_puzzle_action()
        ));
        group.add_action(&solve_puzzle);

        let pause_action = gio::SimpleAction::new("pause-resume", None);
        pause_action.connect_activate(clone!(
            #[weak(rename_to = mself)]
            self,
            move |_, _| mself.pause_resume_action()
        ));
        group.add_action(&pause_action);

        let show_warnings = gio::SimpleAction::new("show-warnings", None);
        show_warnings.connect_activate(clone!(
            #[weak(rename_to = mself)]
            self,
            move |_, _| mself.show_warnings_action()
        ));
        group.add_action(&show_warnings);

        let show_duplicates = gio::SimpleAction::new("show-duplicates", None);
        show_duplicates.connect_activate(clone!(
            #[weak(rename_to = mself)]
            self,
            move |_, _| mself.show_duplicates_action()
        ));
        group.add_action(&show_duplicates);

        self.insert_action_group("game-view", Some(&group));
    }

    #[template_callback]
    fn show_puzzle_bg_cb(&self) {
        if let Some(g) = self.imp().game.get() {
            self.set_background_css(g.borrow().puzzle.colors.get_bg_css());
        }
    }

    #[template_callback]
    fn timer_visible_cb(&self) {
        let imp: &imp::HexkudoGameView = self.imp();
        if !imp.play_pause_stack.is_visible()
            && let Some(g) = imp.game.get()
        {
            self.resume(&mut g.borrow_mut());
        }
    }

    // Load the high score boards from the disk
    fn get_highscores(&self) -> HighScores {
        let saver: SaverHighScores = SaverHighScores::new(glib::user_data_dir());
        if let Ok(o) = saver.get_highscores() {
            if let Some(h) = o {
                return h;
            }
        } else {
            // Delete the file in error for trying to resolve the issue for the next start
            saver.delete_save();
        }
        HighScores::new()
    }

    fn print_current_action(&self) {
        let imp: &imp::HexkudoGameView = self.imp();
        let game = imp
            .game
            .get()
            .expect("Cannot retrieve the game data from the object")
            .borrow();

        if !game.paused {
            self.action_set_enabled("game-view.print-current", false);
            self.action_set_enabled("app.print-multiple", false);
            imp.drawing_area.print_current();
            self.action_set_enabled("game-view.print-current", true);
            self.action_set_enabled("app.print-multiple", true);
        }
    }

    fn zoom_out_action(&self) {
        let imp = self.imp();
        let current_zoom: draw::ZoomLevel = self.zoom_level();
        let new_zoom: draw::ZoomLevel = current_zoom.zoom_out();

        if new_zoom != current_zoom {
            self.set_zoom_level(new_zoom);
            if new_zoom.is_fully_zoomed_out() {
                self.action_set_enabled("game-view.zoom-out", false);
            } else {
                self.action_set_enabled("game-view.zoom-out", true);
            }
            self.action_set_enabled("game-view.zoom-in", true);
            imp.drawing_area.set_zoom_level(new_zoom);
            imp.drawing_area.queue_draw();
        }
    }

    fn zoom_in_action(&self) {
        let imp: &imp::HexkudoGameView = self.imp();
        let current_zoom: draw::ZoomLevel = self.zoom_level();
        let new_zoom: draw::ZoomLevel = current_zoom.zoom_in();

        if new_zoom != current_zoom {
            self.set_zoom_level(new_zoom);
            if new_zoom.is_fully_zoomed_in() {
                self.action_set_enabled("game-view.zoom-in", false);
            } else {
                self.action_set_enabled("game-view.zoom-in", true);
            }
            self.action_set_enabled("game-view.zoom-out", true);
            imp.drawing_area.set_zoom_level(new_zoom);
            imp.drawing_area.queue_draw();
        }
    }

    fn undo_action(&self) {
        let imp: &imp::HexkudoGameView = self.imp();
        let mut game = imp
            .game
            .get()
            .expect("Cannot retrieve the game data from the object")
            .borrow_mut();

        if !game.solved && !game.paused {
            let player_input = &mut game.player_input;

            player_input.undo();
            self.action_set_enabled("game-view.undo", player_input.undo_len() > 0);
            self.action_set_enabled("game-view.redo", player_input.redo_len() > 0);
            self.hide_popover();
            imp.drawing_area.queue_draw();
        }
    }

    fn redo_action(&self) {
        let imp: &imp::HexkudoGameView = self.imp();
        let mut game = imp
            .game
            .get()
            .expect("Cannot retrieve the game data from the object")
            .borrow_mut();

        if !game.solved && !game.paused {
            let player_input = &mut game.player_input;

            player_input.redo();
            self.action_set_enabled("game-view.undo", player_input.undo_len() > 0);
            self.action_set_enabled("game-view.redo", player_input.redo_len() > 0);
            self.hide_popover();
            imp.drawing_area.queue_draw();
        }
    }

    fn reset_puzzle_action(&self) {
        let imp: &imp::HexkudoGameView = self.imp();
        let mut game = imp
            .game
            .get()
            .expect("Cannot retrieve the game data from the object")
            .borrow_mut();

        if !game.paused {
            game.reset();
            self.sensitive(true, &game);
            self.action_set_enabled("game-view.pause-resume", true);
            imp.drawing_area.queue_draw();
        }
    }

    fn set_checkpoint_action(&self) {
        let imp: &imp::HexkudoGameView = self.imp();
        let mut game = imp
            .game
            .get()
            .expect("Cannot retrieve the game data from the object")
            .borrow_mut();

        if !game.solved && !game.paused {
            game.set_checkpoint();
            self.action_set_enabled("game-view.undo-checkpoint", true);
            let toast: adw::Toast = adw::Toast::new(&gettext("Checkpoint set"));
            toast.set_timeout(2);
            imp.toast_overlay.add_toast(toast);
        }
    }

    fn undo_checkpoint_action(&self) {
        let window: gtk::Window = self.root().unwrap().downcast::<gtk::Window>().unwrap();
        let dialog: adw::AlertDialog = adw::AlertDialog::new(
            Some(&gettext("Undo to Last Checkpoint?")),
            Some(&gettext(
                "Are you sure that you want to undo all actions since the last checkpoint?",
            )),
        );
        dialog.add_response("cancel", &gettext("Cancel"));
        dialog.add_response("undo", &gettext("Undo"));
        dialog.set_response_appearance("undo", adw::ResponseAppearance::Destructive);
        dialog.set_default_response(Some("cancel"));
        dialog.set_close_response("cancel");
        dialog.connect_response(
            None,
            glib::clone!(
                #[weak(rename_to = mself)]
                self,
                move |_w, response_id| {
                    if response_id == "undo" {
                        let imp: &imp::HexkudoGameView = mself.imp();
                        let mut game = imp
                            .game
                            .get()
                            .expect("Cannot retrieve the game data from the object")
                            .borrow_mut();
                        game.undo_checkpoint();
                        mself.action_set_enabled(
                            "game-view.undo-checkpoint",
                            game.checkpoints_len() > 0,
                        );
                        mself
                            .action_set_enabled("game-view.undo", game.player_input.undo_len() > 0);
                        mself
                            .action_set_enabled("game-view.redo", game.player_input.redo_len() > 0);
                        imp.drawing_area.queue_draw();
                    }
                }
            ),
        );
        dialog.present(Some(&window));
    }

    fn solve_current_cell_action(&self) {
        let imp: &imp::HexkudoGameView = self.imp();
        let mut game = imp
            .game
            .get()
            .expect("Cannot retrieve the game data from the object")
            .borrow_mut();

        if !game.solved
            && !game.paused
            && let Some((cid, value)) = game.get_selected_cell_value()
        {
            game.user_has_cheated = true;
            self.set_cell_value(game.deref_mut(), cid, value);
            self.hide_popover();
            imp.drawing_area.queue_draw();
        }
    }

    fn solve_puzzle_action(&self) {
        let imp: &imp::HexkudoGameView = self.imp();
        let mut game = imp
            .game
            .get()
            .expect("Cannot retrieve the game data from the object")
            .borrow_mut();

        if !game.solved && !game.paused {
            game.user_has_cheated = true;
            game.player_input.clear();
            for (i, cid) in game.path.get().clone().iter().enumerate() {
                game.player_input.add_no_undo(*cid, i + 1);
            }
            self.check_completed(game.deref_mut());
            self.hide_popover();
            imp.drawing_area.queue_draw();
        }
    }

    fn show_warnings_action(&self) {
        self.imp().drawing_area.switch_warnings();
    }

    fn show_duplicates_action(&self) {
        self.imp().drawing_area.switch_duplicates();
    }

    fn pause_resume_action(&self) {
        let mut game = self
            .imp()
            .game
            .get()
            .expect("Cannot retrieve the game data from the object")
            .borrow_mut();

        if !game.solved {
            if game.paused {
                self.resume(&mut game);
            } else {
                self.pause(&mut game);
            }
        }
    }

    fn pause(&self, game: &mut Game) {
        let imp: &imp::HexkudoGameView = self.imp();
        let attr_list: pango::AttrList = match imp.paused_label.attributes() {
            None => pango::AttrList::new(),
            Some(a) => a,
        };
        let window: gtk::Window = self.root().unwrap().downcast::<gtk::Window>().unwrap();

        // Adjust the size of the Pause label to the size of the window
        attr_list.change(pango::AttrSize::new_size_absolute(
            (window.default_width() as f32 * 0.125) as i32 * pango::SCALE,
        ));
        imp.paused_label.set_attributes(Some(&attr_list));

        self.hide_popover();
        self.sensitive(false, game);
        self.action_set_enabled("app.new-game", false);
        self.action_set_enabled("app.new-game-same-puzzle", false);

        imp.play_pause_stack.set_visible_child(&*imp.play_button);
        imp.box_paused.set_visible(true);
        game.pause();
        imp.drawing_area.queue_draw();
        imp.resume_button.grab_focus();
    }

    fn resume(&self, game: &mut Game) {
        let imp: &imp::HexkudoGameView = self.imp();

        self.sensitive(true, game);
        self.action_set_enabled("app.new-game", true);
        self.action_set_enabled("app.new-game-same-puzzle", true);

        imp.play_pause_stack.set_visible_child(&*imp.pause_button);
        imp.box_paused.set_visible(false);
        game.resume();
        imp.drawing_area.queue_draw();
    }

    pub fn hide_popover(&self) {
        self.imp().drawing_area.hide_popover();
    }

    fn sensitive(&self, sensitive: bool, game: &Game) {
        self.imp().drawing_area.set_sensitive(sensitive);
        self.action_set_enabled("game-view.set-checkpoint", sensitive);
        self.action_set_enabled("game-view.solve-current-cell", sensitive);
        self.action_set_enabled("game-view.solve-puzzle", sensitive);
        self.action_set_enabled("game-view.reset-puzzle", sensitive);
        self.action_set_enabled("game-view.print-current", sensitive);
        self.action_set_enabled("game-view.show_warnings", sensitive);
        self.action_set_enabled("game-view.show_duplicates", sensitive);
        if sensitive {
            self.action_set_enabled("game-view.undo", game.player_input.undo_len() > 0);
            self.action_set_enabled("game-view.redo", game.player_input.redo_len() > 0);
            self.action_set_enabled("game-view.undo-checkpoint", game.checkpoints_len() > 0);
        } else {
            self.action_set_enabled("game-view.undo", false);
            self.action_set_enabled("game-view.redo", false);
            self.action_set_enabled("game-view.undo-checkpoint", false);
        }
    }

    fn set_title(&self, name: &str, difficulty: puzzles::Difficulty) {
        self.imp().window_title.set_subtitle(
            &formatx!(
                gettext("{puzzle_name} {difficulty} Difficulty"),
                puzzle_name = name,
                difficulty = difficulty
            )
            .unwrap()
            .to_string(),
        );
    }

    fn enable_zoom_actions(&self) {
        let zoom_level: draw::ZoomLevel = self.zoom_level();

        if zoom_level.is_fully_zoomed_out() {
            self.action_set_enabled("game-view.zoom-out", false);
        } else {
            self.action_set_enabled("game-view.zoom-out", true);
        }
        if zoom_level.is_fully_zoomed_in() {
            self.action_set_enabled("game-view.zoom-in", false);
        } else {
            self.action_set_enabled("game-view.zoom-in", true);
        }
    }

    fn set_background_css(&self, css_str: &str) {
        match gdk::Display::default() {
            Some(display) => {
                let imp: &imp::HexkudoGameView = self.imp();
                let style_css_provider: &gtk::CssProvider = imp
                    .style_css_provider
                    .get()
                    .expect("Cannot get the CSS provider");

                if css_str.is_empty() || !imp.show_puzzle_bg.get() {
                    style_css_provider.load_from_string(".game-view { }");
                } else {
                    style_css_provider.load_from_string(&format!(".game-view {{ {css_str} }}"));
                }
                gtk::style_context_add_provider_for_display(
                    &display,
                    style_css_provider,
                    gtk::STYLE_PROVIDER_PRIORITY_USER,
                );
            }
            None => debug!("Cannot get display: skipping CSS background styling"),
        }
    }

    pub fn display_scores(&self, highlight_position: Option<usize>) {
        let imp: &imp::HexkudoGameView = self.imp();
        let window: gtk::Window = self.root().unwrap().downcast::<gtk::Window>().unwrap();
        let puzzle_list = imp
            .puzzle_list
            .get()
            .expect("Cannot retrieve the puzzle list from the object");
        let game = imp
            .game
            .get()
            .expect("Cannot retrieve the game data from the object")
            .borrow();
        let (puzzle_name, difficulty) = if game.puzzle.name.is_empty() {
            (String::from("Classic"), puzzles::Difficulty::Easy)
        } else {
            (game.puzzle.name.clone(), game.puzzle.difficulty)
        };
        let scores: HexkudoScoresDialog =
            HexkudoScoresDialog::new(puzzle_list, &self.get_highscores());

        scores.select_puzzle(&puzzle_name, difficulty, highlight_position);
        scores.present(Some(&window));
    }

    pub fn continue_game(&self) {
        let imp: &imp::HexkudoGameView = self.imp();
        let mut game = imp
            .game
            .get()
            .expect("Cannot retrieve the game data from the object")
            .borrow_mut();

        game.puzzle
            .matrix
            .build_edges()
            .expect("The puzzle definition has an error");

        self.set_title(&game.puzzle.name_i18n[..], game.puzzle.difficulty);
        imp.drawing_area.init_puzzle(&mut game.puzzle);
        imp.drawing_area
            .set_path_from_diamonds_and_map(&game.path, &game.diamonds, &game.map);

        self.enable_zoom_actions();
        self.set_background_css(game.puzzle.colors.get_bg_css());
        self.sensitive(true, &game);
        imp.spinner.set_visible(false);
        if game.paused {
            self.pause(&mut game);
        }
        self.update_error_widget(game.get_errors());
    }

    pub fn set_puzzle(&self, mut puzzle: puzzles::Puzzle) {
        let imp: &imp::HexkudoGameView = self.imp();
        let (sender, receiver) =
            async_channel::bounded::<(path::Path, diamond_and_map::DiamondAndMap)>(1);

        imp.spinner.set_visible(true);
        self.sensitive(
            false,
            &imp.game
                .get()
                .expect("Cannot retrieve the game data from the object")
                .borrow_mut(),
        );

        puzzle
            .matrix
            .build_edges()
            .expect("The puzzle definition has an error");
        self.set_title(&puzzle.name_i18n[..], puzzle.difficulty);
        self.update_error_widget(0);

        imp.drawing_area.init_puzzle(&mut puzzle);
        imp.game
            .get()
            .expect("Cannot retrieve the game data from the object")
            .borrow_mut()
            .set_puzzle(&puzzle);

        glib::spawn_future_local(clone!(
            #[strong]
            sender,
            #[strong]
            puzzle,
            async move {
                let (path, m_and_d) = gio::spawn_blocking(move || {
                    let mut random_path: random_path::RandomPath =
                        random_path::RandomPath::new(&puzzle.matrix.edges, &puzzle.matrix.vertexes);

                    // Retrieve a path, map, and diamond from the puzzle's list in case the process
                    // that generates the puzzle or the diamonds takes too long
                    let random: puzzles::PuzzleSampleGame = (puzzle.get_sample_path_fn)();
                    let path: path::Path = path::Path::from_vec(&random.path);
                    let path_len: usize = path.len();
                    let path_first: usize = path
                        .get_first()
                        .expect("Cannot retrieve the first cell in the path");
                    let path_last: usize = path
                        .get_last()
                        .expect("Cannot retrieve the last cell in the path");

                    // Generate a random path
                    match random_path.generate(None) {
                        Err(_) =>
                        // Too long, the generating process gave up
                        {
                            debug!("Too long (path)");
                            (
                                path,
                                diamond_and_map::DiamondAndMap::from_vec(
                                    &random.diamonds,
                                    &random.map,
                                    path_len,
                                    path_first,
                                    path_last,
                                ),
                            )
                        }
                        Ok(p) => {
                            // Generate diamonds and map
                            let mut diamonds: diamonds::Diamond =
                                diamonds::Diamond::new(&random_path.edges, &p);
                            match diamonds.generate_diamonds(&puzzle.matrix.vertexes) {
                                Err(_) =>
                                // Too long, the generating process gave up
                                {
                                    debug!("Too long (diamonds and map)");
                                    (
                                        path,
                                        diamond_and_map::DiamondAndMap::from_vec(
                                            &random.diamonds,
                                            &random.map,
                                            path_len,
                                            path_first,
                                            path_last,
                                        ),
                                    )
                                }
                                Ok(m_and_d) => (p, m_and_d),
                            }
                        }
                    }
                })
                .await
                .expect("Task needs to finish successfully");
                sender
                    .send((path, m_and_d))
                    .await
                    .expect("The channel needs to be open");
            }
        ));

        glib::spawn_future_local(clone!(
            #[weak]
            imp,
            #[weak(rename_to = mself)]
            self,
            async move {
                while let Ok(path_and_diamonds) = receiver.recv().await {
                    let mut game = imp
                        .game
                        .get()
                        .expect("Cannot retrieve the game data from the object")
                        .borrow_mut();
                    let (path, diamond_and_map) = path_and_diamonds;

                    game.set_path(&path, &diamond_and_map);
                    imp.drawing_area.set_path(&path, &diamond_and_map);
                    imp.spinner.set_visible(false);
                    mself.sensitive(true, &game);
                    mself.action_set_enabled("game-view.pause-resume", true);
                }
            }
        ));

        self.enable_zoom_actions();
        self.action_set_enabled("game-view.undo", false);
        self.action_set_enabled("game-view.redo", false);
        self.set_background_css(puzzle.colors.get_bg_css());
    }

    pub fn remove_cell_value(&self, game: &mut Game, cell_id: usize) {
        game.remove_value_from_cell(cell_id);
        self.action_set_enabled("game-view.undo", true);
        self.action_set_enabled("game-view.redo", false);
        self.update_error_widget(game.get_errors());
    }

    pub fn set_cell_value(&self, game: &mut Game, cell_id: usize, cell_value: usize) {
        game.add_value_to_cell(cell_id, cell_value);
        self.action_set_enabled("game-view.undo", true);
        self.action_set_enabled("game-view.redo", false);
        self.check_completed(game);
        self.update_error_widget(game.get_errors());
    }

    pub fn play_again(&self) {
        let puzzle = self
            .imp()
            .game
            .get()
            .expect("Cannot retrieve the game data from the object")
            .borrow()
            .puzzle
            .clone();

        self.set_puzzle(puzzle);
    }

    fn check_completed(&self, game: &mut Game) {
        if !game.is_solved() {
            return;
        }
        let imp: &imp::HexkudoGameView = self.imp();

        game.started = false;
        self.sensitive(false, game);
        self.action_set_enabled("game-view.pause-resume", false);
        // Allow rerunning and printing the puzzle
        self.action_set_enabled("game-view.reset-puzzle", true);
        self.action_set_enabled("game-view.print-current", true);

        let clock_visible: bool = imp.clock_box.is_visible();
        let mut highscore_position: Option<usize> = None;
        let mut highscores: HighScores = self.get_highscores();

        if clock_visible && !game.user_has_cheated {
            highscore_position = highscores.add_score(
                &game.puzzle.name,
                game.puzzle.difficulty,
                game.get_duration(),
                game.get_errors(),
            );
            // Update the clock one more time to ensure that it displays the same value as the
            // high score board
            let (h, m, s) = game.get_duration_hms();
            Self::update_clock_widget(imp, h, m, s);
            if highscore_position.is_some() {
                let saver: SaverHighScores = SaverHighScores::new(glib::user_data_dir());
                match saver.save_highscores(&highscores) {
                    Ok(()) => (),
                    Err(error) => {
                        debug!("Error saving high scores: {error}");
                        // Delete the file in error for trying to resolve the issue for the next start
                        saver.delete_save();
                    }
                }
            }
        }

        let done_dialog: HexkudoDoneDialog =
            HexkudoDoneDialog::new(game.user_has_cheated, clock_visible, highscore_position);
        let window: gtk::Window = self.root().unwrap().downcast::<gtk::Window>().unwrap();

        done_dialog.connect_response(
            None,
            glib::clone!(
                #[weak(rename_to = obj)]
                self,
                move |_w, response_id| {
                    if response_id == "play-again" {
                        obj.play_again();
                    }
                }
            ),
        );
        done_dialog
            .imp()
            .highscore_button
            .connect_clicked(glib::clone!(
                #[weak(rename_to = obj)]
                self,
                move |_w| {
                    obj.display_scores(highscore_position);
                }
            ));
        done_dialog.present(Some(&window));
    }
}
