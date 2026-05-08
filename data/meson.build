desktop_conf = configuration_data()
desktop_conf.set('app-id', application_id)
desktop_conf.set('icon',   application_id)

desktop_in = configure_file(
  input:         'io.github.j0ck4.Wren.desktop.in.in',
  output:        application_id + '.desktop.in',
  configuration: desktop_conf,
)

desktop_file = i18n.merge_file(
  input:       desktop_in,
  output:      application_id + '.desktop',
  type:        'desktop',
  po_dir:      meson.project_source_root() / 'po',
  install:     true,
  install_dir: datadir / 'applications',
)

desktop_validate = find_program('desktop-file-validate', required: false)
if desktop_validate.found()
  test('validate-desktop', desktop_validate, args: [desktop_file])
endif

metainfo_conf = configuration_data()
metainfo_conf.set('app-id', application_id)

metainfo_in = configure_file(
  input:         'io.github.j0ck4.Wren.metainfo.xml.in.in',
  output:        application_id + '.metainfo.xml.in',
  configuration: metainfo_conf,
)

metainfo_file = i18n.merge_file(
  input:       metainfo_in,
  output:      application_id + '.metainfo.xml',
  po_dir:      meson.project_source_root() / 'po',
  install:     true,
  install_dir: datadir / 'metainfo',
)

appstreamcli = find_program('appstreamcli', required: false)
if appstreamcli.found()
  test('validate-metainfo', appstreamcli,
    args: ['validate', '--no-net', '--explain', metainfo_file])
endif

install_data(
  'icons' / 'hicolor' / 'scalable' / 'apps' / 'io.github.j0ck4.Wren.svg',
  install_dir: datadir / 'icons' / 'hicolor' / 'scalable' / 'apps',
  rename: application_id + '.svg',
)
