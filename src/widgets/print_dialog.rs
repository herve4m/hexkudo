/*
Based on GNOME Sudoku at https://gitlab.gnome.org/GNOME/gnome-sudoku

print_dialog.rs

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

//! Dialog for the print multiple puzzles feature.

use std::cmp::Ordering;
use std::collections::HashMap;

use adw::{prelude::*, subclass::prelude::*};
use glib::{Properties, clone};
use gtk::{gio, glib};

use super::print_job::{HexkudoPrintJob, PrintJobParameters};
use super::print_progress::HexkudoPrintProgress;
use crate::generator::diamond_and_map;
use crate::generator::diamonds;
use crate::generator::path;
use crate::generator::puzzles;
use crate::generator::random_path;

mod imp {
    use super::*;
    use std::cell::{Cell, OnceCell, RefCell};

    #[derive(Debug, Default, Properties, gtk::CompositeTemplate)]
    #[properties(wrapper_type = super::HexkudoPrintDialog)]
    #[template(resource = "/io/github/herve4m/Hexkudo/ui/print_dialog.ui")]
    pub struct HexkudoPrintDialog {
        pub window: OnceCell<gtk::Window>,
        pub puzzle_list: OnceCell<Vec<(puzzles::Difficulty, String, puzzles::Puzzle)>>,

        // Properties
        #[property(get, set, builder(puzzles::Difficulty::Easy))]
        pub print_difficulty: Cell<puzzles::Difficulty>,
        #[property(get, set)]
        pub print_puzzle: RefCell<String>,

        // Template widgets
        #[template_child]
        pub n_puzzles: TemplateChild<adw::SpinRow>,
        #[template_child]
        pub n_puzzles_per_page: TemplateChild<adw::SpinRow>,
        #[template_child]
        pub puzzles: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub puzzle_string_list: TemplateChild<gtk::StringList>,
        #[template_child]
        pub solution: TemplateChild<adw::SwitchRow>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for HexkudoPrintDialog {
        const NAME: &'static str = "HexkudoPrintDialog";
        type Type = super::HexkudoPrintDialog;
        type ParentType = adw::Dialog;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for HexkudoPrintDialog {}
    impl WidgetImpl for HexkudoPrintDialog {}
    impl AdwDialogImpl for HexkudoPrintDialog {}
}

glib::wrapper! {
    pub struct HexkudoPrintDialog(ObjectSubclass<imp::HexkudoPrintDialog>)
        @extends gtk::Widget, adw::Dialog,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::ShortcutManager;
}

#[gtk::template_callbacks]
impl HexkudoPrintDialog {
    /// Create the dialog.
    pub fn new(
        settings: &gio::Settings,
        puzzle_list: &HashMap<(String, puzzles::Difficulty), puzzles::Puzzle>,
        window: gtk::Window,
    ) -> Self {
        let obj: HexkudoPrintDialog = glib::Object::builder().build();
        let imp: &imp::HexkudoPrintDialog = obj.imp();

        // GSettings bindings
        let n_puzzles_adj: gtk::Adjustment = imp.n_puzzles.adjustment();
        let n_puzzles_per_page_adj: gtk::Adjustment = imp.n_puzzles_per_page.adjustment();
        let solution: adw::SwitchRow = imp.solution.get();
        settings
            .bind("print-difficulty", &obj, "print-difficulty")
            .build();
        settings.bind("print-solution", &solution, "active").build();
        settings.bind("print-puzzle", &obj, "print-puzzle").build();
        settings
            .bind("print-number", &n_puzzles_adj, "value")
            .build();
        settings
            .bind("print-number-per-page", &n_puzzles_per_page_adj, "value")
            .build();

        // Retrieve the saved settings for the difficulty level and the puzzle name
        let difficulty_setting: puzzles::Difficulty =
            puzzles::Difficulty::from_repr(settings.enum_("print-difficulty"))
                .expect("Cannot retrieve the print difficulty");
        let puzzle_setting: glib::GString = settings.string("print-puzzle");

        // Convert the puzzle list to a list of tuples that get sorted by difficulty and name for
        // using as a model from the puzzle selection combobox list
        let mut puzzles: Vec<(puzzles::Difficulty, String, puzzles::Puzzle)> = puzzle_list
            .iter()
            .map(|k| (k.0.1, k.0.0.clone(), k.1.clone()))
            .collect();
        puzzles.sort_by(|a, b| {
            if a.0 == b.0 {
                if a.2.name_i18n == b.2.name_i18n {
                    return Ordering::Equal;
                }
                if a.2.name_i18n < b.2.name_i18n {
                    return Ordering::Less;
                }
                return Ordering::Greater;
            }
            if a.0 < b.0 {
                return Ordering::Less;
            }
            Ordering::Greater
        });

        // Add the puzzles to the combobox list
        let mut selected_puzzle_index: usize = 0;
        let puzzle_string_list: gtk::StringList = imp.puzzle_string_list.get();
        for (i, puzzle) in puzzles.iter().enumerate() {
            if puzzle.0 == difficulty_setting && puzzle.1 == puzzle_setting {
                selected_puzzle_index = i;
            }
            puzzle_string_list.append(&format!("{} - {}", puzzle.0, puzzle.2.name_i18n));
        }
        imp.puzzles.set_selected(selected_puzzle_index as u32);

        // Save the given puzzle list and GtkWindow to the object
        imp.puzzle_list
            .set(puzzles)
            .expect("Cannot store the puzzle list in the object");
        imp.window
            .set(window)
            .expect("Cannot store the window in the object");

        obj
    }

    /// Callback for the Print button
    #[template_callback]
    fn print_cb(&self, _button: &gtk::Button) {
        let imp: &imp::HexkudoPrintDialog = self.imp();
        let n_puzzles: usize = imp.n_puzzles.adjustment().value() as usize;
        let n_puzzles_per_page: u32 = imp.n_puzzles_per_page.adjustment().value() as u32;
        let solution: bool = imp.solution.is_active();
        let puzzle_id: u32 = imp.puzzles.selected();
        let mut puzzle: (puzzles::Difficulty, String, puzzles::Puzzle) = imp
            .puzzle_list
            .get()
            .expect("Cannot retrieve the puzzle list")[puzzle_id as usize]
            .clone();

        // Save the user provided puzzle selection to GSettings
        self.set_print_difficulty(puzzle.0);
        self.set_print_puzzle(&*puzzle.1);

        // Build the puzzle
        puzzle
            .2
            .matrix
            .build_edges()
            .expect("The puzzle definition has an error");

        // Do not use the dark theme for printing
        puzzle.2.set_dark(false);

        // Close the dialog
        self.close();

        // Show the progress dialog while generating puzzles
        let window: &gtk::Window = imp.window.get().expect("Cannot retrieve the dialog window");
        let progress_dialog: HexkudoPrintProgress = HexkudoPrintProgress::new();
        progress_dialog.present(Some(window));

        let (sender, receiver) =
            async_channel::bounded::<(Vec<path::Path>, Vec<diamond_and_map::DiamondAndMap>)>(1);

        // Generate random path, map, and diamonds
        let progress: gtk::ProgressBar = progress_dialog.imp().progress.get();
        glib::spawn_future_local(clone!(
            #[strong]
            sender,
            #[strong]
            puzzle,
            #[weak]
            progress,
            async move {
                let p = puzzle.2;
                let mut paths: Vec<path::Path> = Vec::with_capacity(n_puzzles);
                let mut d_and_ms: Vec<diamond_and_map::DiamondAndMap> =
                    Vec::with_capacity(n_puzzles);
                let mut i: usize = 0;

                while i < n_puzzles {
                    let (path, diamonds_and_map) = gio::spawn_blocking(clone!(
                        #[strong(rename_to = edges)]
                        p.matrix.edges,
                        #[strong(rename_to = vertexes)]
                        p.matrix.vertexes,
                        move || {
                            let mut random_path: random_path::RandomPath =
                                random_path::RandomPath::new(&edges, &vertexes);

                            // Retrieve a path, diamond, and map from the puzzle's list in case the
                            // process that generates the puzzle or the diamonds takes too long.
                            let random: puzzles::PuzzleSampleGame = (p.get_sample_path_fn)();
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
                                    match diamonds.generate_diamonds(&vertexes) {
                                        Err(_) =>
                                        // Too long, the generating process gave up
                                        {
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
                                        Ok(d_and_m) => (p, d_and_m),
                                    }
                                }
                            }
                        }
                    ))
                    .await
                    .expect("Task needs to finish successfully");

                    i += 1;
                    paths.push(path);
                    d_and_ms.push(diamonds_and_map);

                    // Update the progress dialog
                    progress.set_fraction(i as f64 / n_puzzles as f64);
                }
                sender
                    .send((paths, d_and_ms))
                    .await
                    .expect("The channel needs to be open");
            }
        ));

        // Waiting for the puzzle generation process to complete
        glib::spawn_future_local(clone!(
            #[strong]
            window,
            #[strong]
            puzzle,
            #[weak]
            progress_dialog,
            async move {
                let mut paths: Vec<path::Path> = Vec::new();
                let mut diamonds_and_map: Vec<diamond_and_map::DiamondAndMap> = Vec::new();

                // Waiting for the generation process to complete
                while let Ok(path_and_diamonds) = receiver.recv().await {
                    (paths, diamonds_and_map) = path_and_diamonds;
                }

                // Convert the DiamondAndMap list into two lists of diamonds and maps
                let mut diamonds: Vec<Vec<(usize, usize)>> = Vec::new();
                let mut maps: Vec<Vec<usize>> = Vec::new();

                for dm in diamonds_and_map {
                    let (d, m) = dm.get_diamond_and_map();
                    diamonds.push(d);
                    maps.push(m);
                }

                // Create a print job with the generated puzzles
                let print_job = HexkudoPrintJob::new(PrintJobParameters {
                    window,
                    puzzle: puzzle.2,
                    paths,
                    diamonds,
                    maps,
                    n_puzzles,
                    n_puzzles_per_page,
                    solutions: solution,
                });

                // Close the progress dialog
                if progress_dialog.parent().is_some() {
                    progress_dialog.close();
                }

                // Print
                print_job.print();
            }
        ));
    }
}
