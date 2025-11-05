/*
Based on GNOME Sudoku at https://gitlab.gnome.org/GNOME/gnome-sudoku

preferences_dialog.rs

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

//! Manage the preferences dialog.

use gettextrs::gettext;

use adw::{prelude::*, subclass::prelude::*};
use gtk::{gdk, gio, glib};

use crate::saver::highscores::SaverHighScores;

/// Create a [`gdk::RGBA`] object from a GSettings color parameter.
pub fn get_rgba(settings: &gio::Settings, key: &str) -> gdk::RGBA {
    let variant: glib::Variant = settings.value(key);
    let rgba: Vec<f64> = variant.iter().map(|i| i.get().unwrap()).collect();
    gdk::RGBA::new(
        rgba[0] as f32,
        rgba[1] as f32,
        rgba[2] as f32,
        rgba[3] as f32,
    )
}

mod imp {
    use super::*;
    use std::cell::OnceCell;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/herve4m/Hexkudo/ui/preferences_dialog.ui")]
    pub struct HexkudoPreferencesDialog {
        pub settings: OnceCell<gio::Settings>,

        // Template widgets
        #[template_child]
        pub show_timer: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub show_errors: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub draw_path: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub number_picker_second_click: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub show_warnings: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub show_duplicates: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub default_color_cell_values: TemplateChild<gtk::Switch>,
        #[template_child]
        pub color_cell_values: TemplateChild<gtk::ColorDialogButton>,
        #[template_child]
        pub default_color_cell_wrong: TemplateChild<gtk::Switch>,
        #[template_child]
        pub color_cell_wrong: TemplateChild<gtk::ColorDialogButton>,
        #[template_child]
        pub default_color_cell_bg: TemplateChild<gtk::Switch>,
        #[template_child]
        pub color_cell_bg: TemplateChild<gtk::ColorDialogButton>,
        #[template_child]
        pub default_color_cell_hint_bg: TemplateChild<gtk::Switch>,
        #[template_child]
        pub color_cell_hint_bg: TemplateChild<gtk::ColorDialogButton>,
        #[template_child]
        pub default_color_sel_cell_bg: TemplateChild<gtk::Switch>,
        #[template_child]
        pub color_sel_cell_bg: TemplateChild<gtk::ColorDialogButton>,
        #[template_child]
        pub default_color_cell_borders: TemplateChild<gtk::Switch>,
        #[template_child]
        pub color_cell_borders: TemplateChild<gtk::ColorDialogButton>,
        #[template_child]
        pub default_color_path: TemplateChild<gtk::Switch>,
        #[template_child]
        pub color_path: TemplateChild<gtk::ColorDialogButton>,
        #[template_child]
        pub show_puzzle_bg: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub sel_thick_border: TemplateChild<adw::SwitchRow>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for HexkudoPreferencesDialog {
        const NAME: &'static str = "HexkudoPreferencesDialog";
        type Type = super::HexkudoPreferencesDialog;
        type ParentType = adw::PreferencesDialog;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for HexkudoPreferencesDialog {}
    impl WidgetImpl for HexkudoPreferencesDialog {}
    impl AdwDialogImpl for HexkudoPreferencesDialog {}
    impl PreferencesDialogImpl for HexkudoPreferencesDialog {}
}

glib::wrapper! {
    pub struct HexkudoPreferencesDialog(ObjectSubclass<imp::HexkudoPreferencesDialog>)
        @extends gtk::Widget, adw::Dialog, adw::PreferencesDialog,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

#[gtk::template_callbacks]
impl HexkudoPreferencesDialog {
    /// Create the dialog.
    pub fn new(settings: &gio::Settings) -> Self {
        let obj: HexkudoPreferencesDialog = glib::Object::builder().build();
        let imp: &imp::HexkudoPreferencesDialog = obj.imp();

        let show_timer: adw::SwitchRow = imp.show_timer.get();
        let show_errors: adw::SwitchRow = imp.show_errors.get();
        let draw_path: adw::SwitchRow = imp.draw_path.get();
        let number_picker_second_click: adw::SwitchRow = imp.number_picker_second_click.get();
        let show_warnings: adw::SwitchRow = imp.show_warnings.get();
        let show_duplicates: adw::SwitchRow = imp.show_duplicates.get();
        let default_color_cell_values: gtk::Switch = imp.default_color_cell_values.get();
        let default_color_cell_wrong: gtk::Switch = imp.default_color_cell_wrong.get();
        let default_color_cell_bg: gtk::Switch = imp.default_color_cell_bg.get();
        let default_color_cell_hint_bg: gtk::Switch = imp.default_color_cell_hint_bg.get();
        let default_color_sel_cell_bg: gtk::Switch = imp.default_color_sel_cell_bg.get();
        let default_color_cell_borders: gtk::Switch = imp.default_color_cell_borders.get();
        let default_color_path: gtk::Switch = imp.default_color_path.get();
        let color_cell_values: gtk::ColorDialogButton = imp.color_cell_values.get();
        let color_cell_wrong: gtk::ColorDialogButton = imp.color_cell_wrong.get();
        let color_cell_bg: gtk::ColorDialogButton = imp.color_cell_bg.get();
        let color_sel_cell_bg: gtk::ColorDialogButton = imp.color_sel_cell_bg.get();
        let color_cell_hint_bg: gtk::ColorDialogButton = imp.color_cell_hint_bg.get();
        let color_cell_borders: gtk::ColorDialogButton = imp.color_cell_borders.get();
        let color_path: gtk::ColorDialogButton = imp.color_path.get();
        let show_puzzle_bg: adw::SwitchRow = imp.show_puzzle_bg.get();
        let sel_thick_border: adw::SwitchRow = imp.sel_thick_border.get();

        // GSettings bindings
        settings.bind("show-timer", &show_timer, "active").build();
        settings.bind("show-errors", &show_errors, "active").build();
        settings.bind("draw-path", &draw_path, "active").build();
        settings
            .bind(
                "number-picker-second-click",
                &number_picker_second_click,
                "active",
            )
            .build();
        settings
            .bind("show-warnings", &show_warnings, "active")
            .build();
        settings
            .bind("show-duplicates", &show_duplicates, "active")
            .build();
        settings
            .bind(
                "use-default-color-cell-values",
                &default_color_cell_values,
                "active",
            )
            .build();
        settings
            .bind(
                "use-default-color-cell-wrong",
                &default_color_cell_wrong,
                "active",
            )
            .build();
        settings
            .bind("use-default-color-bg", &default_color_cell_bg, "active")
            .build();
        settings
            .bind(
                "use-default-color-hint-bg",
                &default_color_cell_hint_bg,
                "active",
            )
            .build();
        settings
            .bind(
                "use-default-sel-color-bg",
                &default_color_sel_cell_bg,
                "active",
            )
            .build();
        settings
            .bind(
                "use-default-color-borders",
                &default_color_cell_borders,
                "active",
            )
            .build();
        settings
            .bind("use-default-color-path", &default_color_path, "active")
            .build();
        settings
            .bind("show-puzzle-bg", &show_puzzle_bg, "active")
            .build();
        settings
            .bind("sel-thick-border", &sel_thick_border, "active")
            .build();

        // Initialize the colors in the Preferences dialog from the GSettings values
        let mut rgba: gdk::RGBA = get_rgba(settings, "color-cell-values");
        color_cell_values.set_rgba(&rgba);
        rgba = get_rgba(settings, "color-cell-wrong");
        color_cell_wrong.set_rgba(&rgba);
        rgba = get_rgba(settings, "color-cell-bg");
        color_cell_bg.set_rgba(&rgba);
        rgba = get_rgba(settings, "color-cell-hint-bg");
        color_cell_hint_bg.set_rgba(&rgba);
        rgba = get_rgba(settings, "color-sel-cell-bg");
        color_sel_cell_bg.set_rgba(&rgba);
        rgba = get_rgba(settings, "color-cell-borders");
        color_cell_borders.set_rgba(&rgba);
        rgba = get_rgba(settings, "color-path");
        color_path.set_rgba(&rgba);

        imp.settings
            .set(settings.clone())
            .expect("Cannot store the settings in the object");

        obj
    }

    #[template_callback]
    fn reset_highscores(&self) {
        let window: gtk::Window = self.root().unwrap().downcast::<gtk::Window>().unwrap();
        let dialog: adw::AlertDialog = adw::AlertDialog::new(
            Some(&gettext("Reset the High Score Boards?")),
            Some(&gettext(
                "Are you sure that you want to delete all the high scores?",
            )),
        );
        dialog.add_response("cancel", &gettext("Cancel"));
        dialog.add_response("reset", &gettext("Reset"));
        dialog.set_response_appearance("reset", adw::ResponseAppearance::Destructive);
        dialog.set_default_response(Some("cancel"));
        dialog.set_close_response("cancel");
        dialog.connect_response(
            None,
            glib::clone!(move |_w, response_id| {
                if response_id == "reset" {
                    SaverHighScores::new(glib::user_data_dir()).delete_save();
                }
            }),
        );
        dialog.present(Some(&window));
    }

    // Update a GSettings with the provided color.
    fn set_gsettings(&self, color_widget: gtk::ColorDialogButton, key: &str) {
        let imp: &imp::HexkudoPreferencesDialog = self.imp();
        if let Some(settings) = imp.settings.get() {
            let rgba: gdk::RGBA = color_widget.rgba();
            let variant: glib::Variant = glib::Variant::tuple_from_iter([
                (rgba.red() as f64).to_variant(),
                (rgba.green() as f64).to_variant(),
                (rgba.blue() as f64).to_variant(),
                (rgba.alpha() as f64).to_variant(),
            ]);

            settings
                .set_value(key, &variant)
                .expect("Cannot save the color in GSettings");
        }
    }

    #[template_callback]
    fn on_color_cell_values(&self) {
        let imp: &imp::HexkudoPreferencesDialog = self.imp();
        let color_cell_values: gtk::ColorDialogButton = imp.color_cell_values.get();
        self.set_gsettings(color_cell_values, "color-cell-values");
    }

    #[template_callback]
    fn on_color_cell_wrong(&self) {
        let imp: &imp::HexkudoPreferencesDialog = self.imp();
        let color_cell_wrong: gtk::ColorDialogButton = imp.color_cell_wrong.get();
        self.set_gsettings(color_cell_wrong, "color-cell-wrong");
    }

    #[template_callback]
    fn on_color_cell_bg(&self) {
        let imp: &imp::HexkudoPreferencesDialog = self.imp();
        let color_cell_bg: gtk::ColorDialogButton = imp.color_cell_bg.get();
        self.set_gsettings(color_cell_bg, "color-cell-bg");
    }

    #[template_callback]
    fn on_color_cell_hint_bg(&self) {
        let imp: &imp::HexkudoPreferencesDialog = self.imp();
        let color_cell_hint_bg: gtk::ColorDialogButton = imp.color_cell_hint_bg.get();
        self.set_gsettings(color_cell_hint_bg, "color-cell-hint-bg");
    }

    #[template_callback]
    fn on_color_sel_cell_bg(&self) {
        let imp: &imp::HexkudoPreferencesDialog = self.imp();
        let color_sel_cell_bg: gtk::ColorDialogButton = imp.color_sel_cell_bg.get();
        self.set_gsettings(color_sel_cell_bg, "color-sel-cell-bg");
    }

    #[template_callback]
    fn on_color_cell_borders(&self) {
        let imp: &imp::HexkudoPreferencesDialog = self.imp();
        let color_cell_borders: gtk::ColorDialogButton = imp.color_cell_borders.get();
        self.set_gsettings(color_cell_borders, "color-cell-borders");
    }

    #[template_callback]
    fn on_color_path(&self) {
        let imp: &imp::HexkudoPreferencesDialog = self.imp();
        let color_path: gtk::ColorDialogButton = imp.color_path.get();
        self.set_gsettings(color_path, "color-path");
    }
}
