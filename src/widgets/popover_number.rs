/*
popover_number.rs

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

//! Manage the popover window that display the cell values selection.

use gettextrs::gettext;
use log::debug;
use std::cell::RefCell;
use std::ops::DerefMut;
use std::rc::Rc;

use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::Properties;
use gtk::{Button, gdk, gio, glib};

use crate::game::Game;
use crate::generator::path;
use crate::generator::puzzles;
use crate::widgets::game_view::HexkudoGameView;

mod imp {
    use super::*;
    use std::cell::{Cell, OnceCell};

    #[derive(Debug, Properties, Default, gtk::CompositeTemplate)]
    #[properties(wrapper_type = super::HexkudoPopoverNumber)]
    #[template(resource = "/io/github/herve4m/Hexkudo/ui/popover_number.ui")]
    pub struct HexkudoPopoverNumber {
        pub buttons: RefCell<Vec<Button>>,
        pub clear_button: OnceCell<Button>,
        pub game: OnceCell<Rc<RefCell<Game>>>,

        #[property(get, set)]
        pub number_picker_second_click: Cell<bool>,

        // Template widgets
        #[template_child]
        pub grid: TemplateChild<gtk::Grid>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for HexkudoPopoverNumber {
        const NAME: &'static str = "HexkudoPopoverNumber";
        type Type = super::HexkudoPopoverNumber;
        type ParentType = gtk::Popover;

        fn class_init(klass: &mut Self::Class) {
            debug!("In class_init()");
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            debug!("In instance_init()");
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for HexkudoPopoverNumber {
        fn constructed(&self) {
            self.parent_constructed();

            debug!("In constructed()");
            let b: Button = Button::builder().label(gettext("Clear")).build();
            b.connect_clicked(glib::clone!(
                #[weak(rename_to = obj)]
                self.obj(),
                move |b| {
                    obj.clear_cell(b);
                }
            ));

            self.clear_button
                .set(b)
                .expect("Cannot attach the clear button to the object");
        }

        fn dispose(&self) {
            self.dispose_template();
        }
    }
    impl WidgetImpl for HexkudoPopoverNumber {}
    impl PopoverImpl for HexkudoPopoverNumber {}
}

glib::wrapper! {
    pub struct HexkudoPopoverNumber(ObjectSubclass<imp::HexkudoPopoverNumber>)
        @extends gtk::Widget, gtk::Popover,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Native, gtk::ShortcutManager;
}

impl HexkudoPopoverNumber {
    /// Initialize the object.
    pub fn init(&self, settings: &gio::Settings, game: &Rc<RefCell<Game>>) {
        let imp: &imp::HexkudoPopoverNumber = self.imp();
        imp.game
            .set(Rc::clone(game))
            .expect("Cannot store the game data into the object");

        settings
            .bind(
                "number-picker-second-click",
                self,
                "number-picker-second-click",
            )
            .build();
    }

    pub fn set_puzzle(&self, puzzle: &puzzles::Puzzle) {
        let imp: &imp::HexkudoPopoverNumber = self.imp();
        let grid = &imp.grid;
        let mut buttons = imp.buttons.borrow_mut();
        let clear_button: &Button = imp
            .clear_button
            .get()
            .expect("Cannot retrieve the clear button from the object");

        // Remove all the button widgets from the grid
        while let Some(w) = grid.first_child() {
            grid.remove(&w);
        }

        // Delete all the buttons
        buttons.clear();

        let num_vertexes: usize = puzzle.matrix.vertexes.num_vertexes;
        let columns: i32 = (num_vertexes as f32 - 2.0).sqrt().ceil() as i32;
        let mut c: i32 = 0;
        let mut r: i32 = 0;

        // Create the button widgets and attach them to the grid
        for v in 1..num_vertexes - 1 {
            let label: String = format!("{}", v + 1);
            let button: Button = Button::builder().label(label).build();
            button.add_css_class("numeric");
            button.connect_clicked(glib::clone!(
                #[weak(rename_to = obj)]
                self,
                move |_| {
                    obj.clicked(v + 1);
                }
            ));

            grid.attach(&button, c, r, 1, 1);
            buttons.push(button);
            c += 1;
            if c == columns {
                c = 0;
                r += 1;
            }
        }
        // Attach the Clear button in the last row if space permits. Otherwise, add it to a new row
        if columns - c >= 2 {
            grid.attach(clear_button, c, r, columns - c, 1);
        } else {
            grid.attach(clear_button, 0, r + 1, columns, 1);
        }
    }

    fn get_game_view(&self) -> HexkudoGameView {
        let mut view_widget: gtk::Widget = self.parent().unwrap();
        loop {
            if view_widget.widget_name() == "game_view" {
                return view_widget.downcast::<HexkudoGameView>().unwrap();
            }
            view_widget = view_widget.parent().unwrap();
        }
    }

    // Callback for the Clear button
    fn clear_cell(&self, clear_button: &Button) {
        let imp: &imp::HexkudoPopoverNumber = self.imp();
        let mut game = imp
            .game
            .get()
            .expect("Cannot retrieve the game data from the object")
            .borrow_mut();
        let selected_cell_id: usize = match game.get_selected_cell() {
            Some(cid) => cid,
            None => return,
        };
        let view: HexkudoGameView = self.get_game_view();

        // Remove the value of the selected cell
        view.remove_cell_value(game.deref_mut(), selected_cell_id);
        game.set_selected_cell_value_updated(false);

        clear_button.set_sensitive(false);

        // Update the status of the buttons
        let buttons = imp.buttons.borrow();
        for (i, b) in buttons.iter().enumerate() {
            if game.player_input.contains_value(i + 2) {
                if b.is_sensitive() {
                    b.add_css_class("duplicate");
                }
            } else {
                b.remove_css_class("duplicate");
            }
        }
    }

    // Callback for the buttons
    fn clicked(&self, value: usize) {
        let mut game = self
            .imp()
            .game
            .get()
            .expect("Cannot retrieve the game data from the object")
            .borrow_mut();
        let selected_cell_id: usize = match game.get_selected_cell() {
            Some(cid) => cid,
            None => return,
        };

        self.get_game_view()
            .set_cell_value(game.deref_mut(), selected_cell_id, value);
        self.popdown();
        game.set_selected_cell_value_updated(false);
    }

    /// Make the buttons for the mapped (hint) cells insensitive
    pub fn set_path(&self, path: &path::Path, map: &Vec<usize>) {
        let buttons = self.imp().buttons.borrow();

        for m in map {
            if let Some(v) = path.vertex_index(*m)
                && v != 0
                && v != path.len() - 1
            {
                buttons[v - 1].set_sensitive(false);
            }
        }
    }

    pub fn show(&self, r: gdk::Rectangle, cell_id: usize) {
        let imp: &imp::HexkudoPopoverNumber = self.imp();
        let mut game = imp
            .game
            .get()
            .expect("Cannot retrieve the game data from the object")
            .borrow_mut();

        // Prevent showing the popover if the puzzle is not ready yet
        if game.path.len() == 0 {
            return;
        }

        // Only display the popover if the cell is already selected (second click feature)
        if imp.number_picker_second_click.get()
            && game.get_selected_cell().is_some_and(|cid| cid != cell_id)
        {
            game.set_selected_cell(Some(cell_id));
            return;
        }

        let clear_button: &Button = imp
            .clear_button
            .get()
            .expect("Cannot retrieve the clear button from the object");

        clear_button.set_sensitive(game.player_input.get_value_from_id(cell_id).is_some());

        let buttons = imp.buttons.borrow();
        for (i, b) in buttons.iter().enumerate() {
            if game.player_input.contains_value(i + 2) {
                if b.is_sensitive() {
                    b.add_css_class("duplicate");
                }
            } else {
                b.remove_css_class("duplicate");
            }
        }

        game.set_selected_cell(Some(cell_id));
        self.set_pointing_to(Some(&r));
        self.popup();
        self.grab_focus();
    }

    pub fn hide(&self) {
        self.popdown();
    }
}
