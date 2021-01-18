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
  '-fcf-protection',
  '--param=ssp-buffer-size=4'
]

possible_link_flags = [
  '-Wl,-z,relro',
  '-Wl,-z,now',
  '-Wl,--gc-sections',
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

configure_file(input : 'config.h.in',
  output : 'config.h',
  configuration : conf)

ptr_size = cc.sizeof('void *')

vendor_inc = include_directories('vendor/nolibc', 'src/stdint', 'vendor/printf')

nix_ld = executable('nix-ld',
                    ['src/nix-ld.c'],
                    install : true,
                    install_dir: get_option('libexecdir'),
                    link_args : [
                      '-fno-asynchronous-unwind-tables',
                      '-fno-ident',
                      '-nostdlib',
                      '-static-pie',
                    ],
                    c_args : [
                      '-fno-asynchronous-unwind-tables',
                      '-fno-ident',
                      '-ffreestanding',
                      '-static-pie',
                      '-DPTR_SIZE=' + ptr_size.to_string()
                    ],
                    pie : true,
                    include_directories : vendor_inc)

subdir('test')
