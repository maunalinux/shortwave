project('shortwave', 'rust',
          version: '3.2.0',
    meson_version: '>= 0.59.0',
          license: 'GPL-3.0-or-later',
)

gnome = import('gnome')
i18n = import('i18n')

# Dependencies
dependency('sqlite3', version: '>= 3.20')
dependency('openssl', version: '>= 1.0')
dependency('dbus-1')

dependency('glib-2.0', version: '>= 2.66')
dependency('gio-2.0', version: '>= 2.66')
dependency('gdk-pixbuf-2.0')
dependency('gtk4', version: '>= 4.0.0')
dependency('libadwaita-1', version: '>=1.2.0')
dependency('shumate-1.0', version: '>=1.0.0')

dependency('gstreamer-1.0', version: '>= 1.16')
dependency('gstreamer-base-1.0', version: '>= 1.16')
dependency('gstreamer-audio-1.0', version: '>= 1.16')
dependency('gstreamer-plugins-base-1.0', version: '>= 1.16')
dependency('gstreamer-plugins-bad-1.0', version: '>= 1.16')
dependency('gstreamer-bad-audio-1.0', version: '>= 1.16')

# Initialize project variables
name       = 'Shortwave'
base_id    = 'de.haeckerfelix.Shortwave'
path_id    = '/de/haeckerfelix/Shortwave'
app_id     = base_id
vcs_tag    = run_command('git', 'rev-parse', '--short', 'HEAD').stdout().strip()
profile    = get_option('profile')
datadir    = join_paths(get_option('prefix'), get_option('datadir'))
localedir  = join_paths(get_option('prefix'), get_option('localedir'))
bindir     = join_paths(get_option('prefix'), get_option('bindir'))

# Change app id for development builds
if profile == 'development'
  app_id = '@0@.Devel'.format(app_id)
endif

# Setup pre-commit hook to run checks before changes are getting committed
if profile == 'development'
  message('Setting up git pre-commit hook..')
  run_command('mkdir', '.git/hooks/')
  run_command('cp', '-f', 'build-aux/pre-commit.hook', '.git/hooks/pre-commit')
endif

# Vendor all Rust dependencies for release tarballs
meson.add_dist_script(
  'build-aux/dist-vendor.sh',
  meson.project_source_root(),
  join_paths(meson.project_build_root(), 'meson-dist', meson.project_name() + '-' + meson.project_version())
)

subdir('data')
subdir('src')
subdir('po')
