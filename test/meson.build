example_lib = shared_library('example-lib', [ 'example-lib.c' ])

example_bin = executable('example-bin',
                  ['example-bin.c'],
                  link_with : [ example_lib ])

patched_example_bin = custom_target('patched-example-bin',
  input : [ example_bin ],
  output : [ 'patched-example-bin' ],
  depends : [ example_bin ],
  command : [
    'sh', '-c', 'cp "$0" "$1" && patchelf --set-interpreter "$2" "$1"',
    '@INPUT@', '@OUTPUT@', nix_ld.full_path(),
  ])

interpreter = run_command('sh', '-c', 'patchelf --print-interpreter $(command -v patchelf)',
                          check : true)

sh = find_program('sh')
test('patched-example-lib', sh,
     # only set LD_DEBUG for program to avoid linker debug output from sh
     args : [ '-c', 'LD_DEBUG=libs ' + patched_example_bin.full_path() ],
     env : [
       'NIX_LD_LIBRARY_PATH=.',
       'NIX_LD=' + interpreter.stdout().strip()
     ],
     depends : [ patched_example_bin ])
