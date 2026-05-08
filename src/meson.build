global_conf = configuration_data()
global_conf.set_quoted('APP_ID',          application_id)
global_conf.set_quoted('PKGDATADIR',      pkgdatadir)
global_conf.set_quoted('PROFILE',         profile)
global_conf.set_quoted('VERSION',         version + name_suffix)
global_conf.set_quoted('GETTEXT_PACKAGE', gettext_package)
global_conf.set_quoted('LOCALEDIR',       localedir)

config = configure_file(
  input:         'config.rs.in',
  output:        'config.rs',
  configuration: global_conf,
)

# Mirror generated config.rs into source tree so cargo (run from project root)
# picks it up. Build dir stays the source of truth.
run_command(
  'cp',
  meson.project_build_root() / 'src' / 'config.rs',
  meson.project_source_root() / 'src' / 'config.rs',
  check: true,
)

resources = gnome.compile_resources(
  meson.project_name(),
  'resources.gresource.xml',
  source_dir:        meson.current_source_dir(),
  gresource_bundle:  true,
  install:           true,
  install_dir:       pkgdatadir,
)

cargo_options = [
  '--manifest-path', meson.project_source_root() / 'Cargo.toml',
]

if get_option('buildtype') == 'release'
  rust_profile = 'release'
else
  rust_profile = 'debug'
endif

custom_target(
  'cargo-build',
  build_by_default:   true,
  build_always_stale: true,
  output:             meson.project_name(),
  console:            true,
  install:            true,
  install_dir:        bindir,
  depends:            resources,
  command: [
    cargo_script,
    meson.project_build_root() / 'src',
    rust_profile,
    meson.project_name(),
    '@OUTPUT@',
    cargo_options,
  ],
)
