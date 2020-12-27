project('nix-ld', ['c'],
        version : '1',
        default_options : ['warning_level=2'],
        meson_version : '>= 0.55')

conf = configuration_data()

possible_cc_flags = [
  '-ffast-math',
  '-fno-common',
  '-fdiagnostics-show-option',
  '-fno-strict-aliasing',
  '-fvisibility=hidden',
  '-fstack-protector',
  '-fstack-protector-strong',
  '-fcf-protection',
  '--param=ssp-buffer-size=4'
]

possible_link_flags = [
  '-Wl,-z,relro',
  '-Wl,-z,now',
  '-fstack-protector'
]

cc = meson.get_compiler('c')

if cc.get_id() != 'gcc'
  error('Only gcc is supported as compiler right now, got: ' + cc.get_id())
endif
add_project_arguments(cc.get_supported_arguments(possible_cc_flags), language : 'c')
add_project_link_arguments(cc.get_supported_link_arguments(possible_link_flags), language : 'c')


nix_system = get_option('nix-system')
if nix_system == ''
  error('-Dnix-system is not set')
endif
conf.set_quoted('NIX_SYSTEM', nix_system)

musl_lib = get_option('musl-lib')
if musl_lib == ''
  error('-Dmusl-lib is not set')
endif

musl_includes = get_option('musl-includes')
if musl_includes == ''
  error('-Dmusl-includes is not set')
endif

configure_file(input : 'config.h.in',
  output : 'config.h',
  configuration : conf)
config_inc = include_directories('.')

nix_ld = executable('nix-ld',
                    ['nix-ld.c'],
                    install : true,
                    install_dir: get_option('libexecdir'),
                    link_args : [
                      '-static-pie',
                      '-B' + musl_lib,
                      '-L' + musl_lib,
                    ],
                    c_args : [
                      '-isystem', musl_includes,
                      '-static-pie',
                      '-B' + musl_lib,
                      '-L' + musl_lib,
                    ],
                    pie : true,
                    include_directories : config_inc)

subdir('test')
