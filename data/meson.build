# Icons #
subdir('icons')

# Desktop file
conf = configuration_data()
conf.set('APP_ID', app_id)
conf.set('PKGNAME', meson.project_name())
conf.set('NAME', name)
desktop_file = i18n.merge_file(
  input: configure_file(
    input: '@0@.desktop.in.in'.format(base_id),
    output: '@0@.desktop.in'.format(app_id),
    configuration: conf
  ),
  output: '@0@.desktop'.format(app_id),
  type: 'desktop',
  po_dir: '../po',
  install: true,
  install_dir: join_paths(datadir, 'applications')
)

# Validate desktop file
desktop_utils = find_program('desktop-file-validate', required: false)
if desktop_utils.found()
  test('Validate desktop file', desktop_utils,
    args: [desktop_file]
  )
endif

# Appdata
conf = configuration_data()
conf.set('APP_ID', app_id)
conf.set('PKGNAME', meson.project_name())
conf.set('NAME', name)
appdata_file = i18n.merge_file(
  input: configure_file(
    input: '@0@.metainfo.xml.in.in'.format(base_id),
    output: '@0@.metainfo.xml.in'.format(app_id),
    configuration: conf
  ),
  output: '@0@.metainfo.xml'.format(app_id),
  po_dir: '../po',
  install: true,
  install_dir: join_paths(datadir, 'metainfo')
)

# Validate Appdata
appstream_util = find_program('appstream-util', required: false)
if appstream_util.found()
  test(
    'Validate appdata', appstream_util,
    args: [
      'validate', '--nonet', appdata_file.full_path()
    ]
  )
endif

# GSchema
conf = configuration_data()
conf.set('APP_ID', app_id)
conf.set('PATH_ID', path_id)
conf.set('PKGNAME', meson.project_name())
configure_file(
  input: '@0@.gschema.xml.in'.format(base_id),
  output: '@0@.gschema.xml'.format(app_id),
  configuration: conf,
  install: true,
  install_dir: datadir / 'glib-2.0' / 'schemas'
)

# Validate GSchema
glib_compile_schemas = find_program('glib-compile-schemas', required: false)
if glib_compile_schemas.found()
  test(
    'Validate gschema', glib_compile_schemas,
    args: [
      '--strict', '--dry-run', meson.current_build_dir()
    ]
  )
endif

# DBus service
conf = configuration_data()
conf.set('APP_ID', app_id)
conf.set('PKGNAME', meson.project_name())
conf.set('BINDIR', bindir)
configure_file(
  input: '@0@.service.in'.format(base_id),
  output: '@0@.service'.format(app_id),
  configuration: conf,
  install_dir: join_paths(datadir,'dbus-1', 'services')
)

# Resources
conf = configuration_data()
conf.set('PATH_ID', path_id)
configure_file(
  input: '@0@.gresource.xml.in'.format(base_id),
  output: '@0@.gresource.xml'.format(app_id),
  configuration: conf,
)
gnome.compile_resources(
  app_id,
  join_paths(meson.project_build_root(), 'data', '@0@.gresource.xml'.format(app_id)),
  gresource_bundle: true,
  source_dir: join_paths(meson.project_build_root(), 'data'),
  install: true,
  install_dir: join_paths(datadir, meson.project_name())
)

# Run required post-install steps
gnome.post_install(
  gtk_update_icon_cache: true,
  glib_compile_schemas: true,
  update_desktop_database: true,
)
