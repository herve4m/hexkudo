#
# Blueprint
#

blueprintc = find_program('blueprint-compiler', version: '>= 0.16')

blueprints = custom_target(
  'blueprints',
  input: files(
    'ui/done_dialog.blp',
    'ui/drawing_area.blp',
    'ui/game_view.blp',
    'ui/shortcuts_dialog.blp',
    'ui/menu_button.blp',
    'ui/popover_number.blp',
    'ui/preferences_dialog.blp',
    'ui/print_dialog.blp',
    'ui/print_progress.blp',
    'ui/puzzle_list_item.blp',
    'ui/scores_dialog.blp',
    'ui/scores_dialog_item.blp',
    'ui/select_puzzle_view.blp',
    'ui/start_view.blp',
    'ui/window.blp',
  ),
  output: '.',
  install_dir: '@CURRENT_SOURCE_DIR@',
  command: [blueprintc, 'batch-compile', '@OUTPUT@', '@CURRENT_SOURCE_DIR@', '@INPUT@'],
)


#
# GNOME resources
#

gnome.compile_resources('hexkudo', 'hexkudo.gresource.xml',
      dependencies: blueprints,
  gresource_bundle: true,
           install: true,
       install_dir: get_option('prefix') / get_option('datadir') / meson.project_name()
)


#
# freedesktop.org Desktop file
#

desktop_conf = configuration_data()
desktop_conf.set('icon', application_id)
desktop_file = i18n.merge_file(
        input: configure_file(
                        input: 'io.github.herve4m.Hexkudo.desktop.in.in',
                       output: '@BASENAME@',
                configuration: desktop_conf
               ),
       output: '@0@.desktop'.format(application_id),
         type: 'desktop',
       po_dir: '../po',
      install: true,
  install_dir: get_option('datadir') / 'applications'
)

desktop_utils = find_program('desktop-file-validate', required: false)
if desktop_utils.found()
  test('Validate desktop file', desktop_utils, args: [desktop_file])
endif


#
# freedesktop.org AppStream MetaInfo file
#

appstream_conf = configuration_data()
appstream_conf.set('application_id', application_id)
appstream_conf.set('version', meson.project_version())
appstream_file = i18n.merge_file(
        input: configure_file(
                        input: 'io.github.herve4m.Hexkudo.metainfo.xml.in.in',
                       output: '@BASENAME@',
                configuration: appstream_conf
               ),
       output: '@0@.metainfo.xml'.format(application_id),
       po_dir: '../po',
      install: true,
  install_dir: get_option('datadir') / 'metainfo'
)

appstreamcli = find_program('appstreamcli', required: false, disabler: true)
test('Validate appstream file', appstreamcli,
  args: ['validate', '--no-net', '--explain', appstream_file])


#
# Gio Settings schema file
#

schema_conf = configuration_data()
# schema_conf.set('application_schema_path', application_schema_path)
schema_conf.set('application_id', application_id)
configure_file(
          input: 'io.github.herve4m.Hexkudo.gschema.xml.in',
         output: '@0@.gschema.xml'.format(application_id),
  configuration: schema_conf,
    install_dir: get_option('datadir') / 'glib-2.0' / 'schemas'
)

compile_schemas = find_program('glib-compile-schemas', required: false, disabler: true)
test('Validate schema file', compile_schemas,
  args: ['--strict', '--dry-run', meson.current_source_dir()])


#
# D-Bus service file
#

service_conf = configuration_data()
service_conf.set('bindir', get_option('prefix') / get_option('bindir'))
service_conf.set('application_id', application_id)
configure_file(
  input: 'io.github.herve4m.Hexkudo.service.in',
  output: '@0@.service'.format(application_id),
  configuration: service_conf,
  install_dir: get_option('datadir') / 'dbus-1' / 'services'
)

subdir('icons')
