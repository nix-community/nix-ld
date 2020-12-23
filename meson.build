project('nix-ld', ['c'],
        version : '1',
        default_options : ['warning_level=2'],
        meson_version : '>= 0.47')

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

musl_lib = get_option('musl-lib')
musl_includes = get_option('musl-includes')

if musl_lib == ''
  error('-Dmusl-lib is not set')
endif

if musl_includes == ''
  error('-Dmusl-includes is not set')
endif


nix_ld = executable('nix-ld',
                    ['nix-ld.c'],
                    install : true,
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
                    pie : true)

subdir('test')
