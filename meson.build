project(
  'wren',
  'rust',
  version: '0.1.0',
  meson_version: '>= 1.0.0',
  license: 'GPL-3.0-or-later',
)

i18n  = import('i18n')
gnome = import('gnome')

base_id         = 'io.github.j0ck4.Wren'
version         = meson.project_version()
gettext_package = meson.project_name()

dependency('glib-2.0',     version: '>= 2.76')
dependency('gio-2.0',      version: '>= 2.76')
dependency('gtk4',         version: '>= 4.14')
dependency('libadwaita-1', version: '>= 1.5')

cargo        = find_program('cargo',       required: true)
cargo_script = find_program('build-aux/cargo.sh')

profile = get_option('profile')
if profile == 'development'
  application_id = base_id + '.Devel'
  name_suffix    = ' (Devel)'
else
  application_id = base_id
  name_suffix    = ''
endif

prefix     = get_option('prefix')
bindir     = prefix / get_option('bindir')
datadir    = prefix / get_option('datadir')
localedir  = prefix / get_option('localedir')
pkgdatadir = datadir / meson.project_name()

subdir('data')
subdir('polkit')
subdir('src')

gnome.post_install(
  gtk_update_icon_cache:   true,
  update_desktop_database: true,
)
