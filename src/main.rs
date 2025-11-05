/*
Based on GNOME Sudoku at https://gitlab.gnome.org/GNOME/gnome-sudoku

main.rs

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

mod application;
mod checkpoint;
mod cli_options;
mod config;
mod draw;
mod game;
mod generator;
mod highscores;
mod input_errors;
mod player_input;
mod saver;
mod widgets;

use self::application::HexkudoApplication;

use config::{GETTEXT_PACKAGE, LOCALEDIR, PKGDATADIR};
use gettextrs::{bind_textdomain_codeset, bindtextdomain, textdomain};
use gtk::prelude::*;
use gtk::{gio, glib};

fn main() -> glib::ExitCode {
    // Hexkudo does not use the option parsing feature provided by GApplication. Clap is used
    // instead.
    if let Some(ret) = cli_options::parse() {
        return glib::ExitCode::from(ret);
    }

    // Set up gettext translations
    bindtextdomain(GETTEXT_PACKAGE, LOCALEDIR).expect("Unable to bind the text domain");
    bind_textdomain_codeset(GETTEXT_PACKAGE, "UTF-8")
        .expect("Unable to set the text domain encoding");
    textdomain(GETTEXT_PACKAGE).expect("Unable to switch to the text domain");

    // Load resources
    let resources = gio::Resource::load(PKGDATADIR.to_owned() + "/hexkudo.gresource")
        .expect("Cannot load resources");
    gio::resources_register(&resources);

    // Create a new GtkApplication. The application manages our main loop,
    // application windows, integration with the window manager/compositor, and
    // desktop features such as file opening and single-instance applications.
    let app: HexkudoApplication = HexkudoApplication::new();

    // Because the arguments are processed by Clap, pass an empty argument list to the GApplication
    let args: Vec<String> = Vec::new();
    app.run_with_args(&args)
}
