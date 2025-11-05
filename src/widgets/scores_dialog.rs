/*
Based on GNOME Sudoku at https://gitlab.gnome.org/GNOME/gnome-sudoku

scores_dialog.rs

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

//! Dialog for the high score boards.

use chrono::{DateTime, Local};
use std::cell::Ref;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::ops::Deref;

use adw::{prelude::*, subclass::prelude::*};
use gtk::{
    gio,
    glib::{self, BoxedAnyObject},
};

use crate::generator::puzzles;
use crate::highscores::{HighScores, Score};
use crate::widgets::scores_dialog_item::{Entry, HexkudoScoreItem};

/// Object that represents a puzzle in the puzzle selection combo box.
#[derive(Debug, Clone)]
pub struct APuzzle {
    name: String,
    difficulty: puzzles::Difficulty,
    puzzle: puzzles::Puzzle,
}

mod imp {
    use super::*;
    use std::cell::{Cell, OnceCell, RefCell};

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/herve4m/Hexkudo/ui/scores_dialog.ui")]
    pub struct HexkudoScoresDialog {
        pub puzzle_list: OnceCell<Vec<APuzzle>>,
        pub highscores: OnceCell<HighScores>,
        pub puzzle_name: OnceCell<String>,
        pub difficulty: OnceCell<puzzles::Difficulty>,
        pub position: OnceCell<Option<usize>>,
        pub current_puzzle_name: RefCell<String>,
        pub current_difficulty: Cell<puzzles::Difficulty>,

        // Template widgets
        #[template_child]
        pub headerbar: TemplateChild<adw::HeaderBar>,
        #[template_child]
        pub dropdown: TemplateChild<gtk::DropDown>,
        #[template_child]
        pub toolbar: TemplateChild<adw::ToolbarView>,
        #[template_child]
        pub column_view: TemplateChild<gtk::ColumnView>,
        #[template_child]
        pub view_stack: TemplateChild<adw::ViewStack>,
        #[template_child]
        pub no_score_page: TemplateChild<adw::StatusPage>,
        #[template_child]
        pub view_score_page: TemplateChild<adw::Clamp>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for HexkudoScoresDialog {
        const NAME: &'static str = "HexkudoScoresDialog";
        type Type = super::HexkudoScoresDialog;
        type ParentType = adw::Dialog;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for HexkudoScoresDialog {}
    impl WidgetImpl for HexkudoScoresDialog {}
    impl AdwDialogImpl for HexkudoScoresDialog {}
}

glib::wrapper! {
    pub struct HexkudoScoresDialog(ObjectSubclass<imp::HexkudoScoresDialog>)
        @extends gtk::Widget, adw::Dialog,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::ShortcutManager;
}

#[gtk::template_callbacks]
impl HexkudoScoresDialog {
    /// Create the dialog.
    pub fn new(
        puzzle_list: &HashMap<(String, puzzles::Difficulty), puzzles::Puzzle>,
        highscores: &HighScores,
    ) -> Self {
        let obj: HexkudoScoresDialog = glib::Object::builder().build();
        let imp: &imp::HexkudoScoresDialog = obj.imp();

        // Convert the puzzle list to a list of APuzzle objects that get sorted by difficulty and
        // name for using as a model from the puzzle selection combobox list
        let mut puzzles: Vec<APuzzle> = puzzle_list
            .iter()
            .map(|k| APuzzle {
                name: k.0.0.clone(),
                difficulty: k.0.1,
                puzzle: k.1.clone(),
            })
            .collect();
        puzzles.sort_by(|a, b| {
            if a.difficulty == b.difficulty {
                if a.puzzle.name_i18n == b.puzzle.name_i18n {
                    return Ordering::Equal;
                }
                if a.puzzle.name_i18n < b.puzzle.name_i18n {
                    return Ordering::Less;
                }
                return Ordering::Greater;
            }
            if a.difficulty < b.difficulty {
                return Ordering::Less;
            }
            Ordering::Greater
        });

        // Construct the model that stores the sorted list of puzzles
        let puzzle_string_list: gtk::StringList = gtk::StringList::new(&[""; 0]);
        for a_puzzle in &puzzles {
            puzzle_string_list.append(&format!(
                "{} - {}",
                a_puzzle.difficulty, a_puzzle.puzzle.name_i18n
            ));
        }

        // Save the given puzzle list and the highscores to the object
        imp.puzzle_list
            .set(puzzles)
            .expect("Cannot store the puzzle list in the object");
        imp.highscores
            .set(highscores.clone())
            .expect("Cannot store the highscore table in the object");

        if highscores.is_empty() {
            imp.headerbar.set_show_title(false);
            imp.view_stack.set_visible_child(&*imp.no_score_page);
        } else {
            imp.headerbar.set_show_title(true);
            imp.dropdown.set_model(Some(&puzzle_string_list));
        }

        obj
    }

    /// Specify the puzzle for which the scoreboard should be displayed.
    pub fn select_puzzle(
        &self,
        puzzle_name: &String,
        puzzle_difficulty: puzzles::Difficulty,
        highlight_position: Option<usize>,
    ) {
        let imp: &imp::HexkudoScoresDialog = self.imp();
        let puzzles = imp
            .puzzle_list
            .get()
            .expect("Cannot retrieve the puzzle list");

        imp.puzzle_name
            .set(puzzle_name.clone())
            .expect("Cannot store the puzzle name in the object");
        imp.difficulty
            .set(puzzle_difficulty)
            .expect("Cannot store the puzzle difficulty in the object");
        imp.position
            .set(highlight_position)
            .expect("Cannot store the player's position in the object");

        for (i, a_puzzle) in puzzles.iter().enumerate() {
            if &a_puzzle.name == puzzle_name && a_puzzle.difficulty == puzzle_difficulty {
                imp.dropdown.set_selected(i as u32);
                return;
            }
        }
        imp.view_stack.set_visible_child(&*imp.no_score_page);
    }

    #[template_callback]
    fn select_puzzle_cb(&self) {
        let imp: &imp::HexkudoScoresDialog = self.imp();
        let puzzle_id: u32 = imp.dropdown.selected();
        let puzzle: &APuzzle = &imp
            .puzzle_list
            .get()
            .expect("Cannot retrieve the puzzle list")[puzzle_id as usize];
        let highscores: &HighScores = imp
            .highscores
            .get()
            .expect("Cannot retrieve the highscore table from the object");
        let puzzle_scores: Option<&Vec<Score>> =
            highscores.get_score(&puzzle.name, puzzle.difficulty);

        imp.current_puzzle_name.replace(puzzle.name.clone());
        imp.current_difficulty.set(puzzle.difficulty);
        if puzzle_scores.is_none_or(|score| score.is_empty()) {
            imp.view_stack.set_visible_child(&*imp.no_score_page);
            return;
        }

        let store: gio::ListStore = gio::ListStore::new::<BoxedAnyObject>();
        for (i, score) in puzzle_scores.unwrap().iter().enumerate() {
            store.append(&BoxedAnyObject::new((i, *score)));
        }

        let sel: gtk::SingleSelection = gtk::SingleSelection::new(Some(store));
        let nosel: gtk::NoSelection = gtk::NoSelection::new(Some(sel));
        imp.column_view.set_model(Some(&nosel));
        imp.view_stack.set_visible_child(&*imp.view_score_page);
    }

    fn use_tags(&self, position: usize) -> bool {
        let imp: &imp::HexkudoScoresDialog = self.imp();

        if let Some(pos) = imp
            .position
            .get()
            .expect("Cannot retrieve the player's position from the object")
        {
            let puzzle_name: &String = imp
                .puzzle_name
                .get()
                .expect("Cannot retrieve the puzzle name from the object");
            let difficulty: &puzzles::Difficulty = imp
                .difficulty
                .get()
                .expect("Cannot retrieve the puzzle difficulty from the object");
            let current_puzzle_name = imp.current_puzzle_name.borrow();
            let current_difficulty: puzzles::Difficulty = imp.current_difficulty.get();

            if position == *pos
                && *puzzle_name == *current_puzzle_name.deref()
                && *difficulty == current_difficulty
            {
                return true;
            }
        }
        false
    }

    #[template_callback]
    fn item_setup_cb(&self, listitem: &gtk::ListItem) {
        let row: HexkudoScoreItem = HexkudoScoreItem::default();
        listitem.set_child(Some(&row));
    }

    #[template_callback]
    fn item_setup_date_time_cb(&self, listitem: &gtk::ListItem) {
        let row: HexkudoScoreItem = HexkudoScoreItem::default();
        // Align left
        row.imp().name.set_xalign(0.0);
        listitem.set_child(Some(&row));
    }

    #[template_callback]
    fn item_bind_pos_cb(&self, listitem: &gtk::ListItem) {
        let child: HexkudoScoreItem = listitem.child().and_downcast::<HexkudoScoreItem>().unwrap();
        let entry: BoxedAnyObject = listitem.item().and_downcast::<BoxedAnyObject>().unwrap();
        let r: Ref<(usize, Score)> = entry.borrow();
        let position: usize = r.0 + 1;

        let position_str: String = if self.use_tags(position) {
            format!("<b><big>{position}</big></b>")
        } else {
            format!("{position}")
        };

        let ent: Entry = Entry { name: position_str };
        child.set_entry(&ent);
    }

    #[template_callback]
    fn item_bind_score_cb(&self, listitem: &gtk::ListItem) {
        let child: HexkudoScoreItem = listitem.child().and_downcast::<HexkudoScoreItem>().unwrap();
        let entry: BoxedAnyObject = listitem.item().and_downcast::<BoxedAnyObject>().unwrap();
        let r: Ref<(usize, Score)> = entry.borrow();
        let duration: std::time::Duration = r.1.time;
        let secs: u64 = duration.as_secs();
        let h: u64 = secs / 3600;
        let m: u64 = (secs % 3600) / 60;
        let s: u64 = (secs % 3600) % 60;
        let ms: u32 = duration.subsec_millis() / 10;
        let time_str: String = if h > 0 {
            format!("{h:02}h {m:02}m {s:02}.{ms:02}s")
        } else if m > 0 {
            format!("{m:02}m {s:02}.{ms:02}s")
        } else {
            format!("{s:02}.{ms:02}s")
        };

        let time_str: String = if self.use_tags(r.0 + 1) {
            format!("<b><big>{time_str}</big></b>")
        } else {
            time_str.to_string()
        };

        let ent: Entry = Entry { name: time_str };
        child.set_entry(&ent);
    }

    #[template_callback]
    fn item_bind_errors_cb(&self, listitem: &gtk::ListItem) {
        let child: HexkudoScoreItem = listitem.child().and_downcast::<HexkudoScoreItem>().unwrap();
        let entry: BoxedAnyObject = listitem.item().and_downcast::<BoxedAnyObject>().unwrap();
        let r: Ref<(usize, Score)> = entry.borrow();
        let errors: usize = r.1.errors;

        let error_str: String = if self.use_tags(r.0 + 1) {
            format!("<b><big>{errors}</big></b>")
        } else {
            format!("{errors}")
        };

        let ent: Entry = Entry { name: error_str };
        child.set_entry(&ent);
    }

    #[template_callback]
    fn item_bind_datetime_cb(&self, listitem: &gtk::ListItem) {
        let child: HexkudoScoreItem = listitem.child().and_downcast::<HexkudoScoreItem>().unwrap();
        let entry: BoxedAnyObject = listitem.item().and_downcast::<BoxedAnyObject>().unwrap();
        let r: Ref<(usize, Score)> = entry.borrow();
        let dt: DateTime<Local> = DateTime::from(r.1.when);
        let ent: Entry = Entry {
            name: format!("{}", dt.format("%c")),
        };
        child.set_entry(&ent);
    }
}
